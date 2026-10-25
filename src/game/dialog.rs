//! Dialoge (wenige Worte, viel Ungesagtes) und die Dienste der NPCs. Aktionen sind `Cmd`-Werte,
//! die das Spiel nach der Auswahl ausführt.

use super::data::*;
use super::Game;

#[derive(Clone, Debug, PartialEq)]
pub enum Cmd {
    None,
    EndTalk,
    Talk(usize),
    TalkPages(Vec<String>),
    Recruit(&'static str),
    LevelMenu,
    LevelStat(usize),
    UpgradeWeapon,
    WeaponShop,
    BuyWeapon(usize),
    Shop,
    Buy(&'static str, i64),
    RestMenu(usize),
    TravelMenu(usize),
    TravelTo(usize),
    Stand,
    Legend,
    Map,
    Close,
    Pause,
    Stats,
    Options,
    ToTitle,
}

#[derive(Clone)]
pub struct Choice {
    pub label: String,
    pub cmd: Cmd,
    pub disabled: bool,
}
pub fn ch(label: &str, cmd: Cmd) -> Choice {
    Choice { label: label.into(), cmd, disabled: false }
}

pub struct Script {
    pub pages: Vec<String>,
    pub choices: Vec<Choice>,
}
fn pages(p: &[&str]) -> Vec<String> {
    p.iter().map(|s| s.to_string()).collect()
}

#[derive(Clone)]
pub struct Entry {
    pub label: String,
    pub right: String,
    pub desc: String,
    pub disabled: bool,
    pub cmd: Cmd,
}
pub fn entry(label: &str, right: &str, desc: &str, disabled: bool, cmd: Cmd) -> Entry {
    Entry { label: label.into(), right: right.into(), desc: desc.into(), disabled, cmd }
}

const FIGS: [&str; 4] = ["fire", "ice", "wind", "earth"];

pub fn dialog_for(g: &mut Game, ni: usize) -> Script {
    let def = g.npcs[ni].def.clone();
    let s = &mut g.save;
    let missing = FIGS.iter().filter(|f| !s.mounted.get(**f).copied().unwrap_or(false)).count();
    let n = s.talked.entry(def.id.to_string()).or_insert(0);
    let first = *n == 0;
    *n += 1;
    let bye = ch("Lebewohl", Cmd::EndTalk);
    match def.dialog {
        "herald" => {
            if first {
                return Script {
                    pages: pages(&[
                        "Ah … noch einer, den die See wieder ausgespuckt hat.",
                        "Hör zu, Ertränkter. Nördlich von hier liegt der Schiffshain. Ein Friedhof für Schiffe, die nicht sterben durften.",
                        "Ihre Galionsfiguren wurden geraubt. Vier Könige der Geisterflotte tragen sie in sich: Glut, Frost, Sturm und Tiefe.",
                        "Bring die Seelen zurück an die Buge. Dann öffnet sich der Strudel, und du kannst bis ans Ende der Welt segeln.",
                        "Dein Schiff wartet am Steg. Es steigt aus der Tiefe, wenn du es rufst.",
                    ]),
                    choices: vec![bye],
                };
            }
            if missing == 0 {
                return Script { pages: pages(&["Der Strudel ruft dich. Nördlich des Hains, wo das Wasser schwarz wird.", "Und wenn du am Ende der Welt stehst … grüß den Moloch nicht von mir."]), choices: vec![bye] };
            }
            Script {
                pages: vec![
                    format!("Noch {missing} Galionsfigur{}. Die Buge im Hain warten.", if missing > 1 { "en fehlen" } else { " fehlt" }),
                    "Die Glut liegt im Osten, der Frost im Nordwesten. Rabenwacht im Westen, die Mangroven im Südosten.".into(),
                ],
                choices: vec![bye],
            }
        }
        "maid" => Script {
            pages: if first {
                pages(&["Willkommen am Seelenbaum, Ertränkter.", "Ich hüte die Laternen. Bring mir Perlen, und ich gieße dein Licht stärker."])
            } else if missing > 0 {
                pages(&["Der Baum flüstert. Er vermisst seine Kinder."])
            } else {
                pages(&["Der Baum ist voll. Hörst du es? Das Wasser im Norden singt."])
            },
            choices: vec![
                ch("Stufe aufsteigen", Cmd::LevelMenu),
                ch("Über den Baum", Cmd::TalkPages(pages(&["Der Baum wuchs aus dem Kiel des ersten Schiffes, das die See verschlang.", "Seine Laternen sind die Seelen jener, die nie heimkamen. Auch deine hing einmal hier."]))),
                bye,
            ],
        },
        "smith" => {
            let lvl = s.weapon_level;
            let (p, sh) = upgrade_cost(lvl);
            Script {
                pages: if first { pages(&["Mein Amboss ist aus einer Kanone gegossen. Er hat mehr Männer erschlagen als ich.", "Gezeitensplitter und Perlen. Dann härte ich dir die Klinge."]) } else { pages(&["Was soll es sein?"]) },
                choices: vec![
                    Choice { label: if lvl >= 5 { "Waffe ist vollendet".into() } else { format!("Waffe härten (+{}) · {p} Perlen, {sh} Splitter", lvl + 1) }, cmd: Cmd::UpgradeWeapon, disabled: lvl >= 5 },
                    ch("Waffen", Cmd::WeaponShop),
                    bye,
                ],
            }
        }
        "innkeep" => {
            let crew = crew_talk(s);
            Script {
                pages: if first { pages(&["Rum, Kugeln, Pulver. Und Geschichten, wenn du zuhörst.", "Die Leute hier trinken nicht, um zu vergessen. Sie trinken, um sich zu erinnern."]) } else { pages(&["Noch einen?"]) },
                choices: vec![ch("Handeln", Cmd::Shop), ch("Die Crew", Cmd::TalkPages(crew)), bye],
            }
        }
        _ => {
            if let Some(el) = def.crew {
                let c = crew(el);
                if s.crew.iter().any(|x| x == el) {
                    let hub = g.zone_key == crate::world::Zone::Hain;
                    return Script {
                        pages: vec![format!("Unser Lied heißt „{}“. Wenn wir auf See sind, drück F, und wir singen.", c.song_name), if hub { "Der Rum hier schmeckt nach Teer. Ich mag ihn.".into() } else { "Wir sehen uns im Hain.".into() }],
                        choices: vec![bye],
                    };
                }
                let lines: &[&str] = match el {
                    "fire" => &["Die Glut hat mein Schiff gefressen. Mit allen, die ich kannte.", "Wenn du den Glutkönig suchst … nimm mich mit. Ich will sehen, wie er brennt."],
                    "ice" => &["Ich habe die Sterne über dem Eis gezählt. Neunhundert Nächte lang.", "Du hast ein Schiff? Dann brauchst du jemanden, der den Weg kennt."],
                    "wind" => &["Rabenwacht war einmal ein Hafen. Jetzt ist es ein Nest.", "Ich kann ein Segel flicken, während es reißt. Willst du so jemanden an Bord?"],
                    _ => &["Die Wurzeln hier unten flüstern, wenn man still ist.", "Ich habe Rümpfe aus Treibholz gebaut. Deiner sieht aus, als bräuchte er mich."],
                };
                return Script { pages: pages(lines), choices: vec![ch("Komm an Bord", Cmd::Recruit(c.el)), ch("Später", Cmd::EndTalk)] };
            }
            if def.dialog == "villager" {
                let lines = ["Der Hain war früher voller Lieder.", "Hast du die Laternen gesehen? Jede ist eine Seele.", "Mein Mann fuhr mit der Sturmflotte. Er kam nie zurück.", "Pass auf dich auf, Ertränkter."];
                return Script { pages: vec![lines[(def.x as usize / 7) % 4].into()], choices: vec![bye] };
            }
            Script { pages: pages(&["…"]), choices: vec![bye] }
        }
    }
}

fn crew_talk(s: &Save) -> Vec<String> {
    if s.crew.is_empty() {
        return pages(&["Deine Crew? Du hast keine. Such in den Ländern der Könige, da sitzen Verlorene an jedem Feuer."]);
    }
    let names: Vec<&str> = s.crew.iter().map(|e| crew(e).name).collect();
    vec![format!("Du hast {} von 4 an Bord. {}.", s.crew.len(), names.join(", ")), "Eine volle Crew singt lauter. Und lautes Singen hält die Geister fern.".into()]
}

pub fn level_entries(s: &Save) -> Vec<Entry> {
    let cost = level_cost(s.level);
    (0..5).map(|i| entry(STAT_NAMES[i], &s.stats.get(STAT_KEYS[i]).to_string(), &format!("{} Kosten: {cost} Perlen", STAT_DESC[i]), s.pearls < cost, Cmd::LevelStat(i))).collect()
}
pub fn weapon_entries(s: &Save) -> Vec<Entry> {
    WEAPONS
        .iter()
        .enumerate()
        .map(|(i, w)| {
            let own = s.owned.iter().any(|o| o == w.key);
            let sc = STAT_KEYS.iter().position(|k| *k == w.scale).map(|i| STAT_NAMES[i]).unwrap_or("");
            entry(
                &format!("{}{}", w.name, if s.weapon == w.key { " (geführt)" } else { "" }),
                &if own { "besessen".to_string() } else { format!("{} Perlen", w.price) },
                &format!("Schaden {} · skaliert mit {sc}", w.dmg),
                !own && (w.price == 0 || s.pearls < w.price as i64),
                Cmd::BuyWeapon(i),
            )
        })
        .collect()
}
pub fn shop_entries(s: &Save) -> Vec<Entry> {
    vec![
        entry("Bleikugeln ×6", "150", item("bullets").desc, s.pearls < 150, Cmd::Buy("bullets", 150)),
        entry("Pulverbomben ×3", "300", item("bombs").desc, s.pearls < 300, Cmd::Buy("bombs", 300)),
        entry("Seelenphiole", if s.phials_max >= 4 { "ausverkauft" } else { "2500" }, item("phial").desc, s.pearls < 2500 || s.phials_max >= 4, Cmd::Buy("phial", 2500)),
    ]
}
