//! Render-Welt: Texturen, Pipelines und der Graph-Knoten, der alle Durchgänge zeichnet.

use super::*;
use bevy::render::{
    render_graph::{Node, NodeRunError, RenderGraph, RenderGraphContext, RenderLabel},
    renderer::{RenderAdapter, RenderContext, RenderDevice, RenderQueue},
    view::ExtractedWindows,
    ExtractSchedule, MainWorld, Render, RenderApp, RenderSet,
};
use std::borrow::Cow;
use wgpu::util::DeviceExt;

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
pub struct SwPassLabel;

pub fn build(app: &mut App) {
    let Some(render_app) = app.get_sub_app_mut(RenderApp) else { return };
    render_app
        .init_resource::<Packed>()
        .add_systems(ExtractSchedule, extract_packed)
        .add_systems(Render, prepare.in_set(RenderSet::PrepareResources));
}

pub fn finish(app: &mut App) {
    let Some(render_app) = app.get_sub_app_mut(RenderApp) else { return };
    let dev = render_app.world().resource::<RenderDevice>().clone();
    let adapter = render_app.world().resource::<RenderAdapter>().clone();
    let st = Gpu::new(&dev, &adapter);
    render_app.insert_resource(st);
    let mut graph = render_app.world_mut().resource_mut::<RenderGraph>();
    graph.add_node(SwPassLabel, SwNode);
    graph.add_node_edge(bevy::render::graph::CameraDriverLabel, SwPassLabel);
}

fn extract_packed(mut main: ResMut<MainWorld>, mut p: ResMut<Packed>) {
    let mut src = main.resource_mut::<Packed>();
    std::mem::swap(&mut p.verts, &mut src.verts);
    p.ranges = src.ranges;
    p.g = src.g;
    p.low = src.low;
    p.uploads.clear();
    p.uploads.append(&mut src.uploads);
    if let Some(i) = src.init.take() {
        p.init = Some(i);
    }
}

struct Targets {
    w: u32,
    h: u32,
    alb: wgpu::TextureView,
    nrm: wgpu::TextureView,
    emi: wgpu::TextureView,
    lit: wgpu::TextureView,
    bright: wgpu::TextureView,
    blur_a: wgpu::TextureView,
    blur_b: wgpu::TextureView,
    _tex: Vec<wgpu::Texture>,
}

#[derive(Resource)]
pub struct Gpu {
    hdr: wgpu::TextureFormat,
    ubuf: wgpu::Buffer,
    s_near: wgpu::Sampler,
    s_lin: wgpu::Sampler,
    bgl0: wgpu::BindGroupLayout,
    bgl_scene: wgpu::BindGroupLayout,
    bgl_post: wgpu::BindGroupLayout,
    bg0: wgpu::BindGroup,
    scene_mod: wgpu::ShaderModule,
    post_mod: wgpu::ShaderModule,
    pl_scene: wgpu::PipelineLayout,
    pl_post: wgpu::PipelineLayout,
    p_gbuf: wgpu::RenderPipeline,
    p_decal: wgpu::RenderPipeline,
    p_over_a: wgpu::RenderPipeline,
    p_over_add: wgpu::RenderPipeline,
    p_light: wgpu::RenderPipeline,
    p_bright: wgpu::RenderPipeline,
    p_blur_h: wgpu::RenderPipeline,
    p_blur_v: wgpu::RenderPipeline,
    p_final: Option<(wgpu::TextureFormat, wgpu::RenderPipeline, wgpu::RenderPipeline)>,
    atlas: Option<(wgpu::Texture, wgpu::Texture)>,
    chunks: (wgpu::Texture, wgpu::Texture),
    mask: Option<wgpu::Texture>,
    noise: wgpu::Texture,
    bg_scene: Option<wgpu::BindGroup>,
    targets: Option<Targets>,
    bg_light: Option<wgpu::BindGroup>,
    bg_bright: Option<wgpu::BindGroup>,
    bg_blur_h: Option<wgpu::BindGroup>,
    bg_blur_v: Option<wgpu::BindGroup>,
    bg_final: Option<wgpu::BindGroup>,
    vbuf: wgpu::Buffer,
    vcap: u64,
    ibuf: wgpu::Buffer,
    icap: u32,
    ranges: [(u32, u32); 6],
    ready: bool,
}

const COMMON: &str = include_str!("shaders/common.wgsl");
const SCENE: &str = include_str!("shaders/scene.wgsl");
const POST: &str = include_str!("shaders/post.wgsl");

fn tex_entry(b: u32, dim: wgpu::TextureViewDimension, filter: bool) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding: b,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: filter }, view_dimension: dim, multisampled: false },
        count: None,
    }
}

fn make_tex(d: &wgpu::Device, label: &str, w: u32, h: u32, layers: u32, fmt: wgpu::TextureFormat, rt: bool) -> wgpu::Texture {
    let mut usage = wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST;
    if rt {
        usage |= wgpu::TextureUsages::RENDER_ATTACHMENT;
    }
    d.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d { width: w, height: h, depth_or_array_layers: layers },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: fmt,
        usage,
        view_formats: &[],
    })
}
fn view_arr(t: &wgpu::Texture) -> wgpu::TextureView {
    t.create_view(&wgpu::TextureViewDescriptor { dimension: Some(wgpu::TextureViewDimension::D2Array), ..Default::default() })
}
fn view2(t: &wgpu::Texture) -> wgpu::TextureView {
    t.create_view(&wgpu::TextureViewDescriptor { dimension: Some(wgpu::TextureViewDimension::D2), ..Default::default() })
}

fn vlayout() -> wgpu::VertexBufferLayout<'static> {
    const ATTRS: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![
        0 => Float32x2, 1 => Float32x2, 2 => Unorm8x4, 3 => Uint8x4, 4 => Float32x2
    ];
    wgpu::VertexBufferLayout { array_stride: std::mem::size_of::<Vtx>() as u64, step_mode: wgpu::VertexStepMode::Vertex, attributes: &ATTRS }
}

const PREMUL: wgpu::BlendState = wgpu::BlendState {
    color: wgpu::BlendComponent { src_factor: wgpu::BlendFactor::One, dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha, operation: wgpu::BlendOperation::Add },
    alpha: wgpu::BlendComponent { src_factor: wgpu::BlendFactor::One, dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha, operation: wgpu::BlendOperation::Add },
};
const ADD: wgpu::BlendState = wgpu::BlendState {
    color: wgpu::BlendComponent { src_factor: wgpu::BlendFactor::One, dst_factor: wgpu::BlendFactor::One, operation: wgpu::BlendOperation::Add },
    alpha: wgpu::BlendComponent { src_factor: wgpu::BlendFactor::One, dst_factor: wgpu::BlendFactor::One, operation: wgpu::BlendOperation::Add },
};

#[allow(clippy::too_many_arguments)]
fn pipe(
    d: &wgpu::Device,
    label: &str,
    layout: &wgpu::PipelineLayout,
    module: &wgpu::ShaderModule,
    vs: &str,
    fs: &str,
    with_vb: bool,
    targets: &[Option<wgpu::ColorTargetState>],
) -> wgpu::RenderPipeline {
    let vb = [vlayout()];
    d.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(layout),
        vertex: wgpu::VertexState { module, entry_point: Some(vs), compilation_options: Default::default(), buffers: if with_vb { &vb } else { &[] } },
        fragment: Some(wgpu::FragmentState { module, entry_point: Some(fs), compilation_options: Default::default(), targets }),
        primitive: wgpu::PrimitiveState { topology: wgpu::PrimitiveTopology::TriangleList, cull_mode: None, ..Default::default() },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    })
}

fn ct(f: wgpu::TextureFormat, blend: Option<wgpu::BlendState>, write: wgpu::ColorWrites) -> Option<wgpu::ColorTargetState> {
    Some(wgpu::ColorTargetState { format: f, blend, write_mask: write })
}

const ALB: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
const NRM: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

impl Gpu {
    fn new(dev: &RenderDevice, adapter: &RenderAdapter) -> Self {
        let d = dev.wgpu_device();
        let f16 = adapter.get_texture_format_features(wgpu::TextureFormat::Rgba16Float);
        let hdr = if f16.allowed_usages.contains(wgpu::TextureUsages::RENDER_ATTACHMENT) && f16.flags.contains(wgpu::TextureFormatFeatureFlags::FILTERABLE) {
            wgpu::TextureFormat::Rgba16Float
        } else {
            wgpu::TextureFormat::Rgba8Unorm
        };
        info!("HDR-Format: {:?}", hdr);
        let ubuf = d.create_buffer(&wgpu::BufferDescriptor {
            label: Some("globals"),
            size: std::mem::size_of::<Globals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let s_near = d.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("near"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let s_lin = d.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("lin"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let bgl0 = d.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bgl0"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry { binding: 1, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None },
                wgpu::BindGroupLayoutEntry { binding: 2, visibility: wgpu::ShaderStages::FRAGMENT, ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering), count: None },
            ],
        });
        use wgpu::TextureViewDimension as Dm;
        let bgl_scene = d.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bgl_scene"),
            entries: &[tex_entry(0, Dm::D2Array, true), tex_entry(1, Dm::D2Array, true), tex_entry(2, Dm::D2Array, true), tex_entry(3, Dm::D2Array, true), tex_entry(4, Dm::D2, true), tex_entry(5, Dm::D2, true)],
        });
        let bgl_post = d.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bgl_post"),
            entries: &[tex_entry(0, Dm::D2, true), tex_entry(1, Dm::D2, true), tex_entry(2, Dm::D2, true), tex_entry(3, Dm::D2, true)],
        });
        let bg0 = d.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("bg0"),
            layout: &bgl0,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: ubuf.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&s_near) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&s_lin) },
            ],
        });
        let scene_src = format!("{COMMON}\n{SCENE}");
        let post_src = format!("{COMMON}\n{POST}");
        let scene_mod = d.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("scene"), source: wgpu::ShaderSource::Wgsl(Cow::Owned(scene_src)) });
        let post_mod = d.create_shader_module(wgpu::ShaderModuleDescriptor { label: Some("post"), source: wgpu::ShaderSource::Wgsl(Cow::Owned(post_src)) });
        let pl_scene = d.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("pl_scene"), bind_group_layouts: &[&bgl0, &bgl_scene], push_constant_ranges: &[] });
        let pl_post = d.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some("pl_post"), bind_group_layouts: &[&bgl0, &bgl_post], push_constant_ranges: &[] });
        let all = wgpu::ColorWrites::ALL;
                let p_gbuf = pipe(d, "gbuf", &pl_scene, &scene_mod, "vs_world", "fs_gbuf", true, &[ct(ALB, None, all), ct(NRM, None, all), ct(hdr, None, all)]);
        let p_decal = pipe(
            d,
            "decal",
            &pl_scene,
            &scene_mod,
            "vs_world",
            "fs_decal",
            true,
            &[ct(ALB, Some(wgpu::BlendState::ALPHA_BLENDING), wgpu::ColorWrites::COLOR)],
        );
        let p_over_a = pipe(d, "over_a", &pl_scene, &scene_mod, "vs_world", "fs_overlay", true, &[ct(hdr, Some(PREMUL), all)]);
        let p_over_add = pipe(d, "over_add", &pl_scene, &scene_mod, "vs_world", "fs_overlay", true, &[ct(hdr, Some(ADD), all)]);
        let p_light = pipe(d, "light", &pl_post, &post_mod, "vs_full", "fs_light", false, &[ct(hdr, None, all)]);
        let p_bright = pipe(d, "bright", &pl_post, &post_mod, "vs_full", "fs_bright", false, &[ct(hdr, None, all)]);
        let p_blur_h = pipe(d, "blur_h", &pl_post, &post_mod, "vs_full", "fs_blur_h", false, &[ct(hdr, None, all)]);
        let p_blur_v = pipe(d, "blur_v", &pl_post, &post_mod, "vs_full", "fs_blur_v", false, &[ct(hdr, None, all)]);
        let chunks = (
            make_tex(d, "chunk_col", CHUNK as u32, CHUNK as u32, CHUNK_SLOTS, ALB, false),
            make_tex(d, "chunk_nrm", CHUNK as u32, CHUNK as u32, CHUNK_SLOTS, NRM, false),
        );
        let noise = make_tex(d, "noise", 256, 256, 1, NRM, false);
        let vcap = 4096 * 4 * std::mem::size_of::<Vtx>() as u64;
        let vbuf = d.create_buffer(&wgpu::BufferDescriptor { label: Some("verts"), size: vcap, usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
        let (ibuf, icap) = make_ibuf(d, 4096);
        Gpu {
            hdr,
            ubuf,
            s_near,
            s_lin,
            bgl0,
            bgl_scene,
            bgl_post,
            bg0,
            scene_mod,
            post_mod,
            pl_scene,
            pl_post,
            p_gbuf,
            p_decal,
            p_over_a,
            p_over_add,
            p_light,
            p_bright,
            p_blur_h,
            p_blur_v,
            p_final: None,
            atlas: None,
            chunks,
            mask: None,
            noise,
            bg_scene: None,
            targets: None,
            bg_light: None,
            bg_bright: None,
            bg_blur_h: None,
            bg_blur_v: None,
            bg_final: None,
            vbuf,
            vcap,
            ibuf,
            icap,
            ranges: [(0, 0); 6],
            ready: false,
        }
    }
}

fn make_ibuf(d: &wgpu::Device, quads: u32) -> (wgpu::Buffer, u32) {
    let mut idx: Vec<u32> = Vec::with_capacity(quads as usize * 6);
    for q in 0..quads {
        let b = q * 4;
        idx.extend_from_slice(&[b, b + 1, b + 2, b, b + 2, b + 3]);
    }
    let buf = d.create_buffer_init(&wgpu::util::BufferInitDescriptor { label: Some("idx"), contents: bytemuck::cast_slice(&idx), usage: wgpu::BufferUsages::INDEX });
    (buf, quads)
}

fn post_bg(d: &wgpu::Device, l: &wgpu::BindGroupLayout, v: [&wgpu::TextureView; 4]) -> wgpu::BindGroup {
    d.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("post_bg"),
        layout: l,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(v[0]) },
            wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(v[1]) },
            wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(v[2]) },
            wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(v[3]) },
        ],
    })
}

fn prepare(mut gpu: ResMut<Gpu>, mut p: ResMut<Packed>, dev: Res<RenderDevice>, queue: Res<RenderQueue>, windows: Res<ExtractedWindows>) {
    let d = dev.wgpu_device();
    let gpu = &mut *gpu;
    let mut scene_dirty = false;
    if let Some(init) = p.init.take() {
        let layers = init.atlas_pages.max(2);
        let ac = make_tex(d, "atlas_col", sw_art::atlas::PAGE as u32, sw_art::atlas::PAGE as u32, layers, ALB, false);
        let an = make_tex(d, "atlas_nrm", sw_art::atlas::PAGE as u32, sw_art::atlas::PAGE as u32, layers, NRM, false);
        gpu.atlas = Some((ac, an));
        gpu.mask = Some(make_tex(d, "mask", init.mask_w.max(1), init.mask_h.max(1), 1, NRM, false));
        scene_dirty = true;
    }
    // Hochladen
    for u in p.uploads.drain(..) {
        let tex = match u.tex {
            Tex::AtlasCol => gpu.atlas.as_ref().map(|a| &a.0),
            Tex::AtlasNrm => gpu.atlas.as_ref().map(|a| &a.1),
            Tex::ChunkCol => Some(&gpu.chunks.0),
            Tex::ChunkNrm => Some(&gpu.chunks.1),
            Tex::Mask => gpu.mask.as_ref(),
            Tex::Noise => Some(&gpu.noise),
        };
        let Some(tex) = tex else { continue };
        queue.write_texture(
            wgpu::TexelCopyTextureInfo { texture: tex, mip_level: 0, origin: wgpu::Origin3d { x: u.x, y: u.y, z: u.layer }, aspect: wgpu::TextureAspect::All },
            &u.data,
            wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(u.w * 4), rows_per_image: Some(u.h) },
            wgpu::Extent3d { width: u.w, height: u.h, depth_or_array_layers: 1 },
        );
    }
    if scene_dirty {
        if let (Some(a), Some(m)) = (&gpu.atlas, &gpu.mask) {
            let v = [view_arr(&a.0), view_arr(&a.1), view_arr(&gpu.chunks.0), view_arr(&gpu.chunks.1), view2(m), view2(&gpu.noise)];
            gpu.bg_scene = Some(d.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("bg_scene"),
                layout: &gpu.bgl_scene,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&v[0]) },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&v[1]) },
                    wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&v[2]) },
                    wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(&v[3]) },
                    wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::TextureView(&v[4]) },
                    wgpu::BindGroupEntry { binding: 5, resource: wgpu::BindingResource::TextureView(&v[5]) },
                ],
            }));
        }
    }
    // Renderziele (Low-Res) bei Größenänderung neu
    let (w, h) = (p.low.0.max(16), p.low.1.max(16));
    if gpu.targets.as_ref().map(|t| t.w != w || t.h != h).unwrap_or(true) {
        let hdr = gpu.hdr;
        let t_alb = make_tex(d, "g_alb", w, h, 1, ALB, true);
        let t_nrm = make_tex(d, "g_nrm", w, h, 1, NRM, true);
        let t_emi = make_tex(d, "g_emi", w, h, 1, hdr, true);
        let t_lit = make_tex(d, "lit", w, h, 1, hdr, true);
        let (bw, bh) = ((w / 2).max(1), (h / 2).max(1));
        let t_br = make_tex(d, "bright", bw, bh, 1, hdr, true);
        let t_ba = make_tex(d, "blur_a", bw, bh, 1, hdr, true);
        let t_bb = make_tex(d, "blur_b", bw, bh, 1, hdr, true);
        let t = Targets {
            w,
            h,
            alb: view2(&t_alb),
            nrm: view2(&t_nrm),
            emi: view2(&t_emi),
            lit: view2(&t_lit),
            bright: view2(&t_br),
            blur_a: view2(&t_ba),
            blur_b: view2(&t_bb),
            _tex: vec![t_alb, t_nrm, t_emi, t_lit, t_br, t_ba, t_bb],
        };
        gpu.bg_light = Some(post_bg(d, &gpu.bgl_post, [&t.alb, &t.nrm, &t.emi, &t.alb]));
        gpu.bg_bright = Some(post_bg(d, &gpu.bgl_post, [&t.lit, &t.lit, &t.lit, &t.lit]));
        gpu.bg_blur_h = Some(post_bg(d, &gpu.bgl_post, [&t.bright, &t.bright, &t.bright, &t.bright]));
        gpu.bg_blur_v = Some(post_bg(d, &gpu.bgl_post, [&t.blur_a, &t.blur_a, &t.blur_a, &t.blur_a]));
        gpu.bg_final = Some(post_bg(d, &gpu.bgl_post, [&t.lit, &t.blur_b, &t.lit, &t.lit]));
        gpu.targets = Some(t);
    }
    // Endbild-Pipelines für das Fensterformat
    if let Some(win) = windows.primary.and_then(|e| windows.get(&e)) {
        if let Some(fmt) = win.swap_chain_texture_format {
            let fmt = fmt.add_srgb_suffix();
            if gpu.p_final.as_ref().map(|f| f.0 != fmt).unwrap_or(true) {
                let fin = pipe(d, "final", &gpu.pl_post, &gpu.post_mod, "vs_full", "fs_final", false, &[ct(fmt, None, wgpu::ColorWrites::ALL)]);
                let ui = pipe(d, "ui", &gpu.pl_scene, &gpu.scene_mod, "vs_ui", "fs_ui", true, &[ct(fmt, Some(PREMUL), wgpu::ColorWrites::ALL)]);
                gpu.p_final = Some((fmt, fin, ui));
            }
        }
    }
    // Vertices
    let bytes: &[u8] = bytemuck::cast_slice(&p.verts);
    if bytes.len() as u64 > gpu.vcap {
        gpu.vcap = (bytes.len() as u64 * 3 / 2).next_power_of_two();
        gpu.vbuf = d.create_buffer(&wgpu::BufferDescriptor { label: Some("verts"), size: gpu.vcap, usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST, mapped_at_creation: false });
    }
    let quads = (p.verts.len() / 4) as u32;
    if quads > gpu.icap {
        let (b, c) = make_ibuf(d, (quads * 3 / 2).next_power_of_two());
        gpu.ibuf = b;
        gpu.icap = c;
    }
    if !bytes.is_empty() {
        queue.write_buffer(&gpu.vbuf, 0, bytes);
    }
    queue.write_buffer(&gpu.ubuf, 0, bytemuck::bytes_of(&p.g));
    gpu.ranges = p.ranges;
    gpu.ready = gpu.bg_scene.is_some() && gpu.targets.is_some() && gpu.p_final.is_some();
}

struct SwNode;

fn att<'a>(v: &'a wgpu::TextureView, clear: Option<wgpu::Color>) -> Option<wgpu::RenderPassColorAttachment<'a>> {
    Some(wgpu::RenderPassColorAttachment {
        view: v,
        resolve_target: None,
        ops: wgpu::Operations { load: match clear { Some(c) => wgpu::LoadOp::Clear(c), None => wgpu::LoadOp::Load }, store: wgpu::StoreOp::Store },
    })
}

impl Node for SwNode {
    fn run(&self, _g: &mut RenderGraphContext, rc: &mut RenderContext, world: &World) -> Result<(), NodeRunError> {
        let gpu = world.resource::<Gpu>();
        if !gpu.ready {
            return Ok(());
        }
        let windows = world.resource::<ExtractedWindows>();
        let Some(win) = windows.primary.and_then(|e| windows.get(&e)) else { return Ok(()) };
        let Some(swap) = &win.swap_chain_texture_view else { return Ok(()) };
        let t = gpu.targets.as_ref().unwrap();
        let (fmt_pipe, ui_pipe) = {
            let f = gpu.p_final.as_ref().unwrap();
            (&f.1, &f.2)
        };
        let enc = rc.command_encoder();
        let black = wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
        let r = gpu.ranges;
        let draw = |pass: &mut wgpu::RenderPass, range: (u32, u32)| {
            if range.1 > 0 {
                pass.draw_indexed(range.0 * 6..(range.0 + range.1) * 6, 0, 0..1);
            }
        };
        // 1) G-Puffer: Boden, dann Schatten (nur Farbe), dann Welt
        let gpass = |enc: &mut wgpu::CommandEncoder, clear: bool, range: (u32, u32)| {
            let c = |col: wgpu::Color| if clear { Some(col) } else { None };
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("gbuf"),
                color_attachments: &[att(&t.alb, c(black)), att(&t.nrm, c(wgpu::Color { r: 0.5, g: 0.5, b: 0.0, a: 0.0 })), att(&t.emi, c(black))],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_bind_group(0, &gpu.bg0, &[]);
            pass.set_bind_group(1, gpu.bg_scene.as_ref().unwrap(), &[]);
            pass.set_vertex_buffer(0, gpu.vbuf.slice(..));
            pass.set_index_buffer(gpu.ibuf.slice(..), wgpu::IndexFormat::Uint32);
            pass.set_pipeline(&gpu.p_gbuf);
            if range.1 > 0 {
                pass.draw_indexed(range.0 * 6..(range.0 + range.1) * 6, 0, 0..1);
            }
        };
        gpass(enc, true, r[0]);
        if r[1].1 > 0 {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("decal"),
                color_attachments: &[att(&t.alb, None)],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_bind_group(0, &gpu.bg0, &[]);
            pass.set_bind_group(1, gpu.bg_scene.as_ref().unwrap(), &[]);
            pass.set_vertex_buffer(0, gpu.vbuf.slice(..));
            pass.set_index_buffer(gpu.ibuf.slice(..), wgpu::IndexFormat::Uint32);
            pass.set_pipeline(&gpu.p_decal);
            draw(&mut pass, r[1]);
        }
        gpass(enc, false, r[2]);
        // 2) Licht + Überlagerung
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("light"),
                color_attachments: &[att(&t.lit, Some(black))],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_bind_group(0, &gpu.bg0, &[]);
            pass.set_bind_group(1, gpu.bg_light.as_ref().unwrap(), &[]);
            pass.set_pipeline(&gpu.p_light);
            pass.draw(0..3, 0..1);
            pass.set_bind_group(1, gpu.bg_scene.as_ref().unwrap(), &[]);
            pass.set_vertex_buffer(0, gpu.vbuf.slice(..));
            pass.set_index_buffer(gpu.ibuf.slice(..), wgpu::IndexFormat::Uint32);
            pass.set_pipeline(&gpu.p_over_a);
            draw(&mut pass, r[3]);
            pass.set_pipeline(&gpu.p_over_add);
            draw(&mut pass, r[4]);
        }
        // 3) Bloom
        let full = |enc: &mut wgpu::CommandEncoder, target: &wgpu::TextureView, pipe: &wgpu::RenderPipeline, bg: &wgpu::BindGroup| {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("post"),
                color_attachments: &[att(target, Some(black))],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_bind_group(0, &gpu.bg0, &[]);
            pass.set_bind_group(1, bg, &[]);
            pass.set_pipeline(pipe);
            pass.draw(0..3, 0..1);
        };
        full(enc, &t.bright, &gpu.p_bright, gpu.bg_bright.as_ref().unwrap());
        full(enc, &t.blur_a, &gpu.p_blur_h, gpu.bg_blur_h.as_ref().unwrap());
        full(enc, &t.blur_b, &gpu.p_blur_v, gpu.bg_blur_v.as_ref().unwrap());
        // 4) Endbild + UI auf den Bildschirm
        {
            let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("final"),
                color_attachments: &[att(swap, Some(black))],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_bind_group(0, &gpu.bg0, &[]);
            pass.set_bind_group(1, gpu.bg_final.as_ref().unwrap(), &[]);
            pass.set_pipeline(fmt_pipe);
            pass.draw(0..3, 0..1);
            pass.set_bind_group(1, gpu.bg_scene.as_ref().unwrap(), &[]);
            pass.set_vertex_buffer(0, gpu.vbuf.slice(..));
            pass.set_index_buffer(gpu.ibuf.slice(..), wgpu::IndexFormat::Uint32);
            pass.set_pipeline(ui_pipe);
            draw(&mut pass, r[5]);
        }
        Ok(())
    }
}
