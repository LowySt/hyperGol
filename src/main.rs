#![allow(warnings, unused)]

use isahc::prelude::*;

use futures::try_join;
use futures::join;
use futures::executor::block_on;
use std::time::Instant;
use std::thread;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufWriter;
use std::io::prelude::*;
use bytebuffer::ByteBuffer;
use widestring::Utf32String;
use byte_slice_cast::*;
use byteorder::{ByteOrder, LittleEndian};

// My shit
pub mod skills;
use crate::skills::*;

pub mod pages;
use crate::pages::*;

pub mod parse_util;
use crate::parse_util::*;

pub mod vec_cache;
use crate::vec_cache::*;

/* Year 2022
Final Time with everything on CachedIndex, Concurrenr Requests
Elapsed:                      34014 ms

Total Mob Get Pages Time:     24224 ms
Total Mob Create Entity Time:     0 ms
Prepare Mob Entry Time:           0 ms
BufferMob Entry Time:             0 ms
Total NPC Get Pages Time:      6323 ms
Total NPC Create Entity Time:     1 ms
*/

static BLACKLIST: [&str; 61] = 
["/wiki/Dinosauro", "/wiki/Gremlin", "/wiki/Formian", "/wiki/Leshy", "/wiki/Meccanico", "/wiki/Popolo_Oscuro",
 "/wiki/Diavolo", "/wiki/Demone", "/wiki/Pesce", "/wiki/Agathion", "/wiki/Angelo", "/wiki/Arconte",
 "/wiki/Asura", "/wiki/Azata", "/wiki/Daemon", "/wiki/Div", "/wiki/Drago", "/wiki/Eone", "/wiki/Inevitabile",
 "/wiki/Kami", "/wiki/Kyton", "/wiki/Oni", "/wiki/Protean", "/wiki/Psicopompo", "/wiki/Qlippoth", "/wiki/Rakshasa",
 "/wiki/Razza_Predatrice", "/wiki/Sahkil", "/wiki/Spirito_della_Casa", "/wiki/Tsukumogami", "/wiki/Wysp", 
 "/wiki/Carnideforme", "/wiki/Golem", "/wiki/Robot", "/wiki/Thriae", "/wiki/Alveare", "/wiki/Drago_Esterno", 
 "/wiki/Sfinge", "/wiki/Caccia_Selvaggia", "/wiki/Ombra_Notturna", "/wiki/Manasaputra", "/wiki/Siktempora", "/wiki/Calamit%C3%A0", "/wiki/Demodand", "/wiki/Linnorm", "/wiki/Araldo", "/wiki/Colosso", "/wiki/Behemoth", 
 "/wiki/Idra", "/wiki/Signore_dei_Qlippoth", "/wiki/Progenie_di_Rovagug", "/wiki/Kaiju", "/wiki/Arcidiavolo", 
 "/wiki/Signore_dei_Demoni", "/wiki/Signore_Empireo", "/wiki/Grande_Antico", "/wiki/Kyton#Kyton_Demagogo", 
 "/wiki/Cavaliere_dell%27Apocalisse", "/wiki/File:35.png", "/wiki/Idolo", "/wiki/Azi"];

static NPC_BLACKLIST: [&str; 71] = 
["/wiki/Umano", "/wiki/Adepto", "/wiki/Combattente", "/wiki/Esperto", "/wiki/Mezzelfo", "/wiki/Popolano",
 "/wiki/Aristocratico", "/wiki/Halfling", "/wiki/Nano", "/wiki/Chierico", "/wiki/Ladro", "/wiki/Bardo",
 "/wiki/Ranger", "/wiki/Monaco", "/wiki/Druido", "/wiki/Mago", "/wiki/Barbaro", "/wiki/Guerriero", "/wiki/Paladino",
 "/wiki/Mezzorco", "/wiki/Stregone", "/wiki/Elfo", "/wiki/Gnomo", "/wiki/Investigatore", "/wiki/Cacciatore",
 "/wiki/Scaldo", "/wiki/Intrepido", "/wiki/Sacerdote_Guerriero", "/wiki/Inquisitore", "/wiki/Razze/Hobgoblin",
 "/wiki/Pistolero", "/wiki/Razze/Tengu", "/wiki/Cavaliere", "/wiki/Razze/Goblin", "/wiki/Fattucchiere",
 "/wiki/Alchimista", "/wiki/Attaccabrighe", "/wiki/Medium", "/wiki/Razze/Tiefling", "/wiki/Vigilante", "/wiki/Antipaladino", "/wiki/Cineta", "/wiki/Ghast", "/wiki/Predatore", "/wiki/Ninja", "/wiki/Magus", "/wiki/Razze/Orco", "/wiki/Minotauro", "/wiki/Occultista", "/wiki/Iracondo_di_Stirpe", "/wiki/Sciamano", 
 "/wiki/Assassino", "/wiki/Duellante", "/wiki/Ombra_Danzante", "/wiki/Mistificatore_Arcano", "/wiki/Arciere_Arcano",
 "/wiki/Maestro_del_Sapere", "/wiki/Leshy_Fungo", "/wiki/Discepolo_dei_Draghi", "/wiki/Guerriero_Arcano", "/wiki/Trovatore", "/wiki/Teurgo_Mistico", "/wiki/Parapsichico", "/wiki/Arcanista", "/wiki/Nixie_di_Palude",
 "/wiki/Mesmerista", "/wiki/Convocatore", "/wiki/Cacciatore_di_Taglie", "/wiki/Razze/Vishkanya", "/wiki/Oracolo",
 "/wiki/Spiritista"];

fn check_path_against_blacklist(p: &str) -> bool {
	for v in BLACKLIST {
		if p == v { return true; }
	}
    
    for v in NPC_BLACKLIST {
		if p == v { return true; }
	}
    
    return false;
}

fn get_path_from_slice(begin: &str) -> Option<&str> {
	let begin_of_path = begin.get(5..);
	if begin_of_path.is_none() { println!("[ERROR] No begin"); panic!(); }
    
	let end_index = begin_of_path.unwrap().find("\"");
	if end_index.is_none() { println!("[ERROR] No end index"); panic!(); }
    
	let resource = begin_of_path.unwrap().get(..end_index.unwrap());
	if resource.is_none() { println!("[ERROR] resource"); panic!(); }
    
	let resource = resource.unwrap();
	
	if  resource.contains("redlink=1") ||
		//resource.contains("Tipo") || 
		//resource.contains("Sottotipo") ||
		resource.contains("Archetipo") { return None; }
    
	if check_path_against_blacklist(resource) == true { return None; }
    
	return Some(resource);
}

macro_rules! check_unwrap
{
    ($val:expr, $file_idx:expr, $name:expr) =>
    {
        {
            if $val.is_none() { println!("IDX: {}, name: {}", $file_idx, $name); panic!(); }
            let res = $val.unwrap();
            res
        }
    }
}

#[derive(Debug)]
struct Mob_Entry
{
    origin     : u32,
    short_desc : u32,
    ac         : u32,
    pf         : u32,
    st         : u32,
    rd         : u32,
    ri         : u32,
    def_cap    : u32,
    melee      : u32,
    ranged     : u32,
    spec_atk   : u32,
    psych      : u32,
    magics     : u32,
    spells     : u32,
    skills     : [u32; 24],
    racial_mods: u32,
    spec_qual  : u32,
    specials   : [u32; 24],
    org        : u32,
    treasure   : u32,
    desc       : u32,
    source     : u32,
    
    name       : u16,
    gs         : u16,
    pe         : u16,
    align      : u16,
    typ        : u16,
    sub        : [u16; 8],
    arch       : [u16; 4],
    size       : u16,
    init       : u16,
    senses     : [u16; 8],
    perception : u16,
    aura       : u16,
    immunities : [u16; 16],
    resistances: [u16; 16],
    weaknesses : [u16; 16],
    speed      : u16,
    space      : u16,
    reach      : u16,
    str        : u16,
    dex        : u16,
    con        : u16,
    int        : u16,
    wis        : u16,
    cha        : u16,
    bab        : u16,
    cmb        : u16,
    cmd        : u16,
    talents    : [u16; 24],
    lang       : [u16; 24],
    env        : u16,
}

static mut create_mob_entry_prepare_time: u128 = 0u128;
static mut create_mob_entry_buffer_time: u128  = 0u128;

fn create_mob_entry(cache: &mut VectorCache,
                    bufs: &mut Buffer_Context, mut page: Mob_Page, file_idx: usize) -> Mob_Entry
{
    //NOTE:Profiliing -------------
    let cme_now = Instant::now();
    //-----------------------------
    
    let head_check    = ["GS", "PE:"];
    let head_arr      = fill_array_from_available(&page.header, &head_check);
    
    //NOTE: Just helping golarion out. If missing shit, I'm gonna report it to fix it.
    //if page.desc.is_empty() { println!("IDX: {}, Name: {}", file_idx, head_arr[0]); }
    
    let class_check   = ["Allineamento: ", "Categoria: ", "(", ")"];
    let mut class_arr = fill_array_from_available(&page.class, &class_check);
    
    //NOTE: Manually fix the category block
    let mut subtypes_count = 0;
    let mut arch_count     = 0;
    if class_arr[3].is_empty()
    {
        let size_idx = class_arr[2].rfind(" ");
        if size_idx.is_none()
        { println!("{:#?}\nMaybe error? size_idx is missing. How can it be missing?", page.page_addr); panic!(); }
        
        let size = class_arr[2].get(size_idx.unwrap()..);
        if size.is_none() { println!("How can I not get size??"); panic!(); }
        
        class_arr[5] = size.unwrap().trim();
        let type_plus_arch = class_arr[2].get(..size_idx.unwrap()).unwrap();
        
        let arch_begin_idx = type_plus_arch.find('[');
        let arch_end_idx = type_plus_arch.find(']');
        
        if arch_end_idx.is_some() && arch_begin_idx.is_some()
        {
            let arch = type_plus_arch.get(arch_begin_idx.unwrap()..arch_end_idx.unwrap()).unwrap();
            class_arr[2] = type_plus_arch.get(..arch_begin_idx.unwrap()).unwrap().trim();
            class_arr[4] = arch.trim();
            arch_count = flatten_str_list(&mut class_arr, 4, ", ");
        }
        else
        {
            class_arr[2] = type_plus_arch.trim();
        }
    }
    else 
    {
        let arch_end_idx = class_arr[4].find(']');
        if arch_end_idx.is_some()
        {
            let arch = class_arr[4].get(1..arch_end_idx.unwrap()).unwrap();
            let size = class_arr[4].get(arch_end_idx.unwrap()+1..).unwrap().trim();
            class_arr[4] = arch;
            
            class_arr.push(size);
            
            arch_count = flatten_str_list(&mut class_arr, 4, ", ");
        }
        else { class_arr.insert(4, GLOBAL_NULL); } //NOTE: We need to preserve the empty spot of the arch
        
        subtypes_count = flatten_str_list(&mut class_arr, 3, ", ");
    }
    
    let subtypes_off = if subtypes_count > 0 { subtypes_count - 1 } else { 0 };
    let arch_off     = if arch_count > 0  { (arch_count - 1) + subtypes_off } else { subtypes_off };
    
    //NOTE We differentiate all senses, and from perception we only keep the value
    let misc_check    = ["Sensi:", "Aura:"];
    let mut misc_arr  = fill_array_from_available(&page.misc, &misc_check);
    
    //NOTE: Manually fix all misc
    let mut senses_count = 0;
    let mut perception_off = 0;
    if !misc_arr[1].is_empty() 
    { 
        //NOTE: Grab initiative value
        misc_arr[0] = check_unwrap!(misc_arr[0].get(12..), file_idx, head_arr[0]);
        
        //NOTE: Look for perception at the end of senses
        //TODO: If the space after perception is missing, we can't separate it.
        let index = misc_arr[1].rfind("Percezione ");
        if index.is_some()
        {
            let index = index.unwrap();
            let senses_value = misc_arr[1].get(..index);
            if senses_value.is_none() { println!("[ERROR] Senses/Perception Fucked\n"); panic!(); }
            
            let perception_value = misc_arr[1].get(index+11..);
            if perception_value.is_none() { println!("[ERROR] Senses/Perception Fucked\n"); panic!(); }
            
            misc_arr[1] = senses_value.unwrap();
            misc_arr.insert(2, perception_value.unwrap());
            perception_off = 1;
        }
        
        senses_count = flatten_str_list(&mut misc_arr, 1, ", ");
    }
    else
    {
        //NOTE: Look for perception at the end of initiative
        let index = misc_arr[0].rfind("Percezione ");
        if index.is_some()
        {
            let index = index.unwrap();
            let init_value = misc_arr[0].get(12..index);
            
            let perception_value = misc_arr[0].get(index+11..);
            let perception_value = check_unwrap!(perception_value, file_idx, head_arr[0]);
            
            misc_arr[0] = check_unwrap!(init_value, file_idx, head_arr[0]);
            misc_arr.insert(2, perception_value);
            perception_off = 1;
        }
        else
        {
            misc_arr[0] = check_unwrap!(misc_arr[0].get(12..), file_idx, head_arr[0]);
        }
    }
    
    let defense_check   = ["PF: ", "Tiri Salvezza: ", "RD: ", "RI: ", "Immunità: ", 
                           "Resistenze: ", "Capacità Difensive: ", "Debolezze: "];
    let mut defense_arr = fill_array_from_available(&page.defense, &defense_check);
    
    let mut immunities_count = 0;
    if !defense_arr[5].is_empty()
    {
        immunities_count = flatten_str_list(&mut defense_arr, 5, ", ");
    }
    
    let mut res_count = 0;
    let res_offset = if immunities_count > 0 { immunities_count - 1 } else { 0 };
    if !defense_arr[6+res_offset].is_empty()
    {
        res_count = flatten_str_list(&mut defense_arr, 6+res_offset, ", ");
    }
    
    let mut weak_count = 0;
    let weak_offset = if res_count > 0 { (res_count - 1) + res_offset } else { res_offset };
    if !defense_arr[8+weak_offset].is_empty()
    {
        weak_count = flatten_str_list(&mut defense_arr, 8+weak_offset, ", ");
    }
    
    //NOTE: Manually fix AC
    defense_arr[0] = defense_arr[0].get(4..).unwrap();
    
    //TODO Maybe further parsing to compress and better separate attacks and spell strings?
    let attack_check   = ["Mischia:", "Distanza:", "Attacchi Speciali:", "Spazio:", "Portata:",
                          "Magia Psichica:", "Capacità Magiche:", "Incantesimi Conosciuti:" ];
    let mut attack_arr = fill_array_from_available(&page.attack, &attack_check);
    
    //NOTE: Manually fix Speed
    attack_arr[0] = attack_arr[0].get(11..).unwrap();
    
    let stats_check   = ["Bonus di Attacco Base:", "BMC:", "DMC:", "Talenti:", "Abilità:",
                         "Linguaggi:", "Modificatori Razziali:", "Qualità Speciali:" ];
    let mut stats_arr = fill_array_from_available(&page.stats, &stats_check);
    
    //NOTE: Manually fix Stats
    stats_arr[0] = stats_arr[0].get(17..).unwrap();
    
    let mut talent_count = 0;
    if !stats_arr[4].is_empty()
    {
        //TODO Fix Arma Focalizzata (Spada, Arco), 
        //     Producing: [Arma Focalizzata (Spada,], [ Arco),]
        talent_count = flatten_str_list(&mut stats_arr, 4, ", ");
    }
    
    //println!("{:#?}", head_arr[0]);
    
    let mut skills_idx  = [0u32; 24];
    let mut skill_count = 0;
    let skill_off = if talent_count > 0 { talent_count - 1 } else { 0 };
    if !stats_arr[5+skill_off].is_empty()
    {
        //skill_count = prepare_skill_str(&mut stats_arr, &page.page_addr, skill_off);
        skills_idx = prepare_skill_str(&mut stats_arr, cache, bufs, &page.page_addr, skill_off);
        
        //println!("{:#?}:\n{:#?}", head_arr[0], skills_idx);
    }
    
    //for ijk in 0..skill_count { println!("\t{:#?}", stats_arr[5+skill_off+ijk]); }
    
    let mut lang_count = 0;
    let lang_off = if skill_count > 0 { (skill_count - 1) + skill_off } else { skill_off };
    if !stats_arr[6+lang_off].is_empty()
    {
        lang_count = flatten_str_list(&mut stats_arr, 6+lang_off, ", ");
    }
    
    //NOTE: At the end, to keep indices correct, we unwrap all stats
    {
        let must_be_six = flatten_str_list(&mut stats_arr, 0, ", ");
        assert!(must_be_six == 6);
        
        stats_arr[0] = stats_arr[0].get(5..).unwrap().trim();
        stats_arr[1] = stats_arr[1].get(9..).unwrap().trim();
        stats_arr[2] = stats_arr[2].get(12..).unwrap().trim();
        stats_arr[3] = stats_arr[3].get(12..).unwrap().trim();
        stats_arr[4] = stats_arr[4].get(8..).unwrap().trim();
        stats_arr[5] = stats_arr[5].get(7..).unwrap().trim();
    }
    
    let mut specials_arr: Vec<&str> = Vec::new();
    let mut el: &str = "";
    let mut next: &str = &page.special;
    loop
    {
        (el, next)  = get_until(next, "\n\n");
        
        if el == GLOBAL_NULL { break; }
        
        //println!("{:#?}\n\n", el);
        specials_arr.push(el);
    }
    //println!("{:#?}", specials_arr);
    
    let ecology_check   = ["Organizzazione:", "Tesoro:"];
    let mut ecology_arr = fill_array_from_available(&page.ecology, &ecology_check);
    
    //NOTE: Manually fix Environment
    ecology_arr[0] = check_unwrap!(ecology_arr[0].get(9..), file_idx, head_arr[0]).trim();
    
    //NOTE: Manually fix source
    page.source = check_unwrap!(page.source.get(7..), file_idx, head_arr[0]).to_string();
    
    
    //NOTE:Profiliing ----------------------------------------------
    unsafe { create_mob_entry_prepare_time += cme_now.elapsed().as_millis(); }
    //--------------------------------------------------------------
    
    
    // ----------------
    //NOTE Start filling the buffers
    // ----------------
    
    
    //NOTE:Profiliing ----------------------------------------------
    let cme_now = Instant::now();
    //--------------------------------------------------------------
    
    
    //Header
    let name_idx = add_entry_if_missing(&mut cache.names, &mut bufs.name, head_arr[0]);
    let gs_idx   = add_entry_if_missing(&mut cache.gs, &mut bufs.gs, head_arr[1]);
    let pe_idx   = add_entry_if_missing(&mut cache.pe, &mut bufs.pe, head_arr[2]);
    
    //Class Info
    let mut subtypes_idx = [0u16; 8];
    let mut arch_idx     = [0u16; 4];
    
    let origin_idx     = add_entry_if_missing_u32(&mut cache.strings, &mut bufs.string, &page.origin);
    let short_desc_idx = add_entry_if_missing_u32(&mut cache.strings, &mut bufs.string, class_arr[0]);
    let align_idx      = add_entry_if_missing(&mut cache.alignment, &mut bufs.alignment, class_arr[1]);
    let type_idx       = add_entry_if_missing(&mut cache.types, &mut bufs.types, class_arr[2]);
    
    for s in 0..subtypes_count
    { subtypes_idx[s]  = add_entry_if_missing(&mut cache.subtypes, &mut bufs.subtypes, class_arr[3+s]); }
    
    for arch in 0..arch_count
    { arch_idx[arch]   = add_entry_if_missing(&mut cache.archetypes, &mut bufs.archetypes, class_arr[4+subtypes_off+arch]); }
    
    let size_idx       = add_entry_if_missing(&mut cache.sizes, &mut bufs.sizes, class_arr[5+arch_off]);
    
    //Misc
    let mut senses_idx = [0u16; 8];
    
    let init_idx       = add_entry_if_missing(&mut cache.numbers, &mut bufs.number, misc_arr[0]);
    
    for s in 0..senses_count
    { senses_idx[s]    = add_entry_if_missing(&mut cache.senses, &mut bufs.senses, misc_arr[1+s]); }
    
    let senses_off     = if senses_count > 0 { senses_count - 1 } else { 0 };
    let perception_idx = add_entry_if_missing(&mut cache.numbers, &mut bufs.number, misc_arr[2+senses_off]);
    
    let aura_off = 2+senses_off+perception_off;
    let aura_idx = add_entry_if_missing(&mut cache.auras, &mut bufs.auras, misc_arr[aura_off]);
    
    //Defense
    let mut immunities_idx  = [0u16; 16];
    let mut resistances_idx = [0u16; 16];
    let mut weaknesses_idx  = [0u16; 16];
    
    //TODO: See if I can move these into the numeric values buffer
    let ac_idx            = add_entry_if_missing_u32(&mut cache.strings, &mut bufs.string, defense_arr[0]);
    let pf_idx            = add_entry_if_missing_u32(&mut cache.strings, &mut bufs.string, defense_arr[1]);
    let st_idx            = add_entry_if_missing_u32(&mut cache.strings, &mut bufs.string, defense_arr[2]);
    let rd_idx            = add_entry_if_missing_u32(&mut cache.strings, &mut bufs.string, defense_arr[3]);
    let ri_idx            = add_entry_if_missing_u32(&mut cache.strings, &mut bufs.string, defense_arr[4]);
    
    for i in 0..immunities_count
    { immunities_idx[i]   = add_entry_if_missing(&mut cache.immunities, &mut bufs.immunities, defense_arr[5+i]); }
    
    for r in 0..res_count
    {
        let off = 6+res_offset+r;
        resistances_idx[r] = add_entry_if_missing(&mut cache.resistances, &mut bufs.resistances, defense_arr[off]);
    }
    
    let def_cap_off = 7 + weak_offset;
    let defensive_cap_idx = add_entry_if_missing_u32(&mut cache.strings, &mut bufs.string, defense_arr[def_cap_off]);
    
    for w in 0..weak_count
    {
        let off = 8+weak_offset+w;
        weaknesses_idx[w] = add_entry_if_missing(&mut cache.weaknesses,&mut bufs.weaknesses, defense_arr[off]);
    }
    
    //Attack
    let speed_idx    = add_entry_if_missing(&mut cache.numbers, &mut bufs.number, attack_arr[0]);
    let melee_idx    = add_entry_if_missing_u32(&mut cache.strings, &mut bufs.string, attack_arr[1]);
    let ranged_idx   = add_entry_if_missing_u32(&mut cache.strings, &mut bufs.string, attack_arr[2]);
    let spec_atk_idx = add_entry_if_missing_u32(&mut cache.strings, &mut bufs.string, attack_arr[3]);
    let space_idx    = add_entry_if_missing(&mut cache.numbers, &mut bufs.number, attack_arr[4]);
    let reach_idx    = add_entry_if_missing(&mut cache.numbers, &mut bufs.number, attack_arr[5]);
    let psych_idx    = add_entry_if_missing_u32(&mut cache.strings, &mut bufs.string, attack_arr[6]);
    let magics_idx   = add_entry_if_missing_u32(&mut cache.strings, &mut bufs.string, attack_arr[7]);
    let spells_idx   = add_entry_if_missing_u32(&mut cache.strings, &mut bufs.string, attack_arr[8]);
    
    //Stats
    let mut talents_idx = [0u16; 24];
    //let mut skills_idx  = [0u32; 24];
    let mut lang_idx    = [0u16; 24];
    
    let str_idx = add_entry_if_missing(&mut cache.numbers, &mut bufs.number, stats_arr[0]);
    let dex_idx = add_entry_if_missing(&mut cache.numbers, &mut bufs.number, stats_arr[1]);
    let con_idx = add_entry_if_missing(&mut cache.numbers, &mut bufs.number, stats_arr[2]);
    let int_idx = add_entry_if_missing(&mut cache.numbers, &mut bufs.number, stats_arr[3]);
    let wis_idx = add_entry_if_missing(&mut cache.numbers, &mut bufs.number, stats_arr[4]);
    let cha_idx = add_entry_if_missing(&mut cache.numbers, &mut bufs.number, stats_arr[5]);
    
    let bab_idx = add_entry_if_missing(&mut cache.numbers, &mut bufs.number, stats_arr[6]);
    let cmb_idx = add_entry_if_missing(&mut cache.numbers, &mut bufs.number, stats_arr[7]);
    let cmd_idx = add_entry_if_missing(&mut cache.numbers, &mut bufs.number, stats_arr[8]);
    
    for t in 0..talent_count
    { talents_idx[t] = add_entry_if_missing(&mut cache.talents, &mut bufs.talents, stats_arr[9+t]); }
    
    //TODO: make skills cool: 25965
    // FIX Herne  Conoscenze (geografia) +6 (+8 nelle foreste)
    //println!("{:#?}:", head_arr[0]);
    /*
    for s in 0..skill_count
    { 
        let num_value_check: &[_] = &['+', '-'];
        let trim_check: &[_] = &[' ', ',', ';', '.'];
        
        if(stats_arr[10+s+skill_off] != "-")
        {
            if let Some((paren_block, paren_begin_idx, paren_end_idx)) = slice_between_inclusive(stats_arr[10+s+skill_off], "(", ")")
            {
                let mut skill_type       = [0u16; 24];
                let mut skill_value      = [-999i16; 24]; //TODO: Use -999 by default as sentinel value
                let mut skill_entry: u16 = 0;
                
                //let mut inside_paren = stats_arr[10+s+skill_off][complex_skill_idx+1..].to_lowercase();
                let mut inside_paren = paren_block.to_lowercase();
                let mut subskill_count = 0;
                for subskill_idx in 0..(Skill_Names::UnoQualsiasi as u16)
                {
                    //TODO: This might be bad. It can make me catch "+12 a saltare" and "+12 saltare"
                    //      But it might also fuck up: "tutte" vs "tutte le altre"
                    let subskill_name = &Skill_Names::from_u16(subskill_idx).as_str();
                    if(inside_paren.contains(subskill_name))
                    {
                        //NOTE: Found one
                        skill_type[subskill_count+1] = subskill_idx;
                        
                        //NOTE: Check for extra value. Is this good enough?
                        if let Some(subskill_val) = inside_paren.find(num_value_check)
                        {
                            let subskill_val_str = inside_paren[subskill_val..].split_once(' ');
                            match(subskill_val_str)
                            {
                                Some(v_str) =>
                                {
                                    skill_value[subskill_count+1] = v_str.0
                                        .trim_matches(trim_check)
                                        .parse::<i16>()
                                        .unwrap_or_else(|_| panic!("{:#?}\n[ERROR] couldn't parse value: {:#?}", head_arr[0], v_str.0));
                                    
                                    inside_paren = v_str.1.replace(subskill_name, "");
                                }
                                
                                None =>
                                {
                                    panic!("Couldn't find end of value: {:#?}", inside_paren);
                                }
                            }
                            
                        }
                        else
                        {
                            inside_paren = inside_paren.replace(subskill_name, "");
                        }
                        
                        subskill_count += 1;
                    }
                }
                
                if(subskill_count == 0)
                {
                    //println!("{:#?}:", head_arr[0]);
                    println!("\t{:#?} Interned", stats_arr[10+s+skill_off]);
                    
                    unsafe { debug_skill_cache_size += 4u32 + stats_arr[10+s+skill_off].len() as u32; }
                }
                else
                {
                    let without_paren = stats_arr[10+s+skill_off].replace(paren_block, "");
                    
                    let mut did_intern = false;
                    let mut name_check = "";
                    if let Some(v) = split_off(&without_paren, "+-")
                    {
                        name_check = v.0.trim();
                        let skill_value_opt = v.1
                            .trim_matches(trim_check)
                            .parse::<i16>();
                        
                        if(skill_value_opt.is_err())
                        {
                            println!("{:#?}:", head_arr[0]);
                            println!("[ERROR] couldn't parse value: {:#?}", stats_arr[10+s+skill_off]);
                            println!("\t{:#?} Interned", stats_arr[10+s+skill_off]);
                            
                            unsafe { debug_skill_cache_size += 4u32 + stats_arr[10+s+skill_off].len() as u32; }
                            
                            //TODO Do the interning
                            did_intern = true;
                        }
                        else
                        {
                            skill_value[0] = skill_value_opt.unwrap();
                        }
                    }
                    else
                    {
                        //NOTE: We just assume there's no bonus because it's +0...
                        name_check = without_paren.trim();
                        skill_value[0] = 0;
                    }
                    
                    if(skill_value[0] != -999)
                    {
                        skill_type[0] = Skill_Names::from_str(name_check);
                        if(skill_type[0] == 0xffff)
                        {
                            println!("\t{:#?} Interned", stats_arr[10+s+skill_off]);
                            
                            unsafe { debug_skill_cache_size += 4u32 + stats_arr[10+s+skill_off].len() as u32; }
                        }
                        else
                        {
                            skill_entry = skill_type[0] | ((skill_value[0] & 0x007F) << 7) as u16;
                            skill_entry |= 0x4000;
                            
                            //println!("\t{:#?}: {:#?} {:#?} -> {:#?}", stats_arr[10+s+skill_off], skill_type[0], skill_value[0], skill_entry);
                            
                            for subskill_idx in 0..subskill_count
                            {
                                //NOTE: All the skill entries
                                //println!("\t\t{:#?} {:#?}", Skill_Names::from_u16(skill_type[subskill_idx+1]), skill_value[subskill_idx+1]);
                            }
                        }
                    }
                    else if(did_intern == false)
                    {
                        //TODO: Probably do the interning?
                        println!("{:#?}:", head_arr[0]);
                        println!("[ERROR] couldn't parse value: {:#?}", without_paren);
                        println!("\t{:#?} Interned", stats_arr[10+s+skill_off]);
                        
                        unsafe { debug_skill_cache_size += 4u32 + stats_arr[10+s+skill_off].len() as u32; }
                    }
                }
            }
            else
            {
                let mut skill_type:  u16 = 0;
                let mut skill_value: i16 = -999;
                let mut skill_entry: u16 = 0;
                
                let mut did_intern = false;
                let mut name_check: &str = "";
                
                if let Some(v) = split_off(stats_arr[10+s+skill_off], "+-")
                {
                    //TODO: In this case there's no paren block. Which means a type without a value should be
                    //      Impossible, and if present it's an error in the source. Change this?
                    name_check = v.0.trim();
                    let skill_value_opt = v.1.trim_matches(trim_check).parse::<i16>();
                    if(skill_value_opt.is_err())
                    {
                        println!("{:#?}:", head_arr[0]);
                        println!("[ERROR] couldn't parse value: {:#?}", stats_arr[10+s+skill_off]);
                        println!("\t{:#?} Interned", stats_arr[10+s+skill_off]);
                        //TODO Do the interning
                        
                        did_intern = true;
                        
                        unsafe { debug_skill_cache_size += 4u32 + stats_arr[10+s+skill_off].len() as u32; }
                    }
                    else
                    {
                        skill_value = skill_value_opt.unwrap();
                    }
                }
                
                if(skill_value != -999)
                {
                    skill_type = Skill_Names::from_str(name_check);
                    if(skill_type == 0xffff)
                    {
                        //println!("{:#?}:", head_arr[0]);
                        //println!("Invalid Type {:#?}", stats_arr[10+s+skill_off]);
                        //TODO: Here I should Intern as Well...
                        //println!("{:#?}:", head_arr[0]);
                        //println!("[ERROR] couldn't parse value: {:#?}", stats_arr[10+s+skill_off]);
                        println!("\t{:#?} Interned", stats_arr[10+s+skill_off]);
                        
                        unsafe { debug_skill_cache_size += 4u32 + stats_arr[10+s+skill_off].len() as u32; }
                    }
                    else
                    {
                        skill_entry = skill_type | ((skill_value & 0x007F) << 7) as u16;
                        //println!("\t{:#?}: {:#?} {:#?} -> {:#?}", stats_arr[10+s+skill_off], skill_type, skill_value, skill_entry);
                    }
                }
                else if(did_intern == false)
                {
                    //TODO: Probably do the interning? Need to do did_intern boolean here as well if I do.
                    println!("{:#?}:", head_arr[0]);
                    println!("[ERROR] couldn't parse value: {:#?}", stats_arr[10+s+skill_off]);
                    println!("\t{:#?} Interned", stats_arr[10+s+skill_off]);
                    
                    unsafe { debug_skill_cache_size += 4u32 + stats_arr[10+s+skill_off].len() as u32; }
                }
            }
        }
        
        skills_idx[s] = add_entry_if_missing_u32(&mut cache.skills, &mut bufs.skills, stats_arr[10+s+skill_off]);
    }
    */
    //println!("");
    
    for l in 0..lang_count
    { lang_idx[l]    = add_entry_if_missing(&mut cache.languages, &mut bufs.languages, stats_arr[11+l+lang_off]); }
    
    let after_lang_off = if lang_count > 0 { (lang_count - 1) + lang_off } else { lang_off };
    let racial_mods  = add_entry_if_missing_u32(&mut cache.strings, &mut bufs.string, stats_arr[12+after_lang_off]);
    let spec_qual    = add_entry_if_missing_u32(&mut cache.strings, &mut bufs.string, stats_arr[13+after_lang_off]);
    
    //All specials
    let mut specials_idx = [0u32; 24];
    for s in 0..specials_arr.len()
    { specials_idx[s] = add_entry_if_missing_u32(&mut cache.specials, &mut bufs.specials, &specials_arr[s]); }
    
    //Ecology
    let env_idx      = add_entry_if_missing(&mut cache.environment, &mut bufs.environment, ecology_arr[0]);
    let org_idx      = add_entry_if_missing_u32(&mut cache.strings, &mut bufs.string, ecology_arr[1]);
    let treasure_idx = add_entry_if_missing_u32(&mut cache.strings, &mut bufs.string, ecology_arr[2]);
    
    //Desc
    let desc_idx     = add_entry_if_missing_u32(&mut cache.strings, &mut bufs.string, &page.desc);
    
    //Source
    let source_idx   = add_entry_if_missing_u32(&mut cache.strings, &mut bufs.string, &page.source);
    
    let mut entry = Mob_Entry {
        name: name_idx,
        gs: gs_idx,
        pe: pe_idx,
        origin: origin_idx,
        short_desc: short_desc_idx,
        align: align_idx,
        typ: type_idx,
        sub: subtypes_idx,
        arch: arch_idx,
        size: size_idx,
        init: init_idx,
        senses: senses_idx,
        perception: perception_idx,
        aura: aura_idx,
        ac: ac_idx,
        pf: pf_idx,
        st: st_idx,
        rd: rd_idx,
        ri: ri_idx,
        immunities: immunities_idx,
        resistances: resistances_idx,
        weaknesses: weaknesses_idx,
        def_cap : defensive_cap_idx,
        speed: speed_idx,
        melee: melee_idx,
        ranged: ranged_idx,
        spec_atk: spec_atk_idx,
        space: space_idx,
        reach: reach_idx,
        psych: psych_idx,
        magics: magics_idx,
        spells: spells_idx,
        str: str_idx,
        dex: dex_idx,
        con: con_idx,
        int: int_idx,
        wis: wis_idx,
        cha: cha_idx,
        bab: bab_idx,
        cmb: cmb_idx,
        cmd: cmd_idx,
        talents: talents_idx,
        skills: skills_idx,
        lang: lang_idx,
        racial_mods: racial_mods,
        spec_qual: spec_qual,
        specials: specials_idx,
        env: env_idx,
        org: org_idx,
        treasure: treasure_idx,
        desc: desc_idx,
        source: source_idx,
    };
    
    //NOTE:Profiling -----------------------------------------------
    unsafe { create_mob_entry_buffer_time += cme_now.elapsed().as_millis(); };
    //--------------------------------------------------------------
    
    return entry;
}

//TODO: Maybe merge this with Mob_Entry in the future??
#[derive(Debug)]
struct NPC_Entry
{
    origin     : u32,
    short_desc : u32,
    ac         : u32,
    pf         : u32,
    st         : u32,
    rd         : u32,
    ri         : u32,
    def_cap    : u32,
    melee      : u32,
    ranged     : u32,
    spec_atk   : u32,
    psych      : u32,
    magics     : u32,
    spells     : u32,
    tactics    : [u32; 3],
    skills     : [u32; 24],
    racial_mods: u32,
    spec_qual  : u32,
    given_equip: u32,
    properties : u32,
    boons      : u32,
    specials   : [u32; 24],
    desc       : u32,
    source     : u32,
    
    name       : u16,
    gs         : u16,
    pe         : u16,
    align      : u16,
    typ        : u16,
    sub        : [u16; 8],
    arch       : [u16; 4],
    size       : u16,
    init       : u16,
    senses     : [u16; 8],
    perception : u16,
    aura       : u16,
    immunities : [u16; 16],
    resistances: [u16; 16],
    weaknesses : [u16; 16],
    speed      : u16,
    space      : u16,
    reach      : u16,
    str        : u16,
    dex        : u16,
    con        : u16,
    int        : u16,
    wis        : u16,
    cha        : u16,
    bab        : u16,
    cmb        : u16,
    cmd        : u16,
    talents    : [u16; 24],
    lang       : [u16; 24],
}

fn create_npc_entry(cache: &mut VectorCache,
                    bufs: &mut Buffer_Context, mut page: NPC_Page, file_idx: usize) -> NPC_Entry
{
    let head_check    = ["GS", "PE:"];
    let head_arr      = fill_array_from_available(&page.header, &head_check);
    
    //NOTE: Just helping golarion out. If missing shit, I'm gonna report it to fix it.
    //if page.desc.is_empty() { println!("IDX: {}, Name: {}", file_idx, head_arr[0]); }
    
    let class_check   = ["Allineamento: ", "Categoria: ", "(", ")"];
    let mut class_arr = fill_array_from_available(&page.class, &class_check);
    /*
    dbg!(page.origin);
    dbg!(class_arr);
    panic!();
    */
    //NOTE: Manually fix the category block
    let mut subtypes_count = 0;
    let mut arch_count     = 0;
    if class_arr[3].is_empty()
    {
        let size_idx = class_arr[2].rfind(" ");
        if size_idx.is_none()
        {
            println!("Can't find size in creature {} in line: {:#?}", head_arr[0], class_arr);
            panic!();
        }
        
        let size = class_arr[2].get(size_idx.unwrap()..);
        if size.is_none() { println!("How can I not get size??"); panic!(); }
        
        class_arr[5] = size.unwrap().trim();
        let type_plus_arch = class_arr[2].get(..size_idx.unwrap()).unwrap();
        
        let arch_begin_idx = type_plus_arch.find('[');
        let arch_end_idx = type_plus_arch.find(']');
        
        if arch_end_idx.is_some() && arch_begin_idx.is_some()
        {
            let arch = type_plus_arch.get(arch_begin_idx.unwrap()..arch_end_idx.unwrap()).unwrap();
            class_arr[2] = type_plus_arch.get(..arch_begin_idx.unwrap()).unwrap().trim();
            class_arr[4] = arch.trim();
            arch_count = flatten_str_list(&mut class_arr, 4, ", ");
        }
        else
        {
            class_arr[2] = type_plus_arch.trim();
        }
    }
    else 
    {
        let arch_end_idx = class_arr[4].find(']');
        if arch_end_idx.is_some()
        {
            let arch = class_arr[4].get(1..arch_end_idx.unwrap()).unwrap();
            let size = class_arr[4].get(arch_end_idx.unwrap()+1..).unwrap().trim();
            class_arr[4] = arch;
            
            class_arr.push(size);
            
            arch_count = flatten_str_list(&mut class_arr, 4, ", ");
        }
        else { class_arr.insert(4, GLOBAL_NULL); } //NOTE: We need to preserve the empty spot of the arch
        
        subtypes_count = flatten_str_list(&mut class_arr, 3, ", ");
    }
    
    let subtypes_off = if subtypes_count > 0 { subtypes_count - 1 } else { 0 };
    let arch_off     = if arch_count > 0  { (arch_count - 1) + subtypes_off } else { subtypes_off };
    
    //NOTE We differentiate all senses, and from perception we only keep the value
    let misc_check    = ["Sensi:", "Aura:"];
    let mut misc_arr  = fill_array_from_available(&page.misc, &misc_check);
    
    //NOTE: Manually fix all misc
    let mut senses_count = 0;
    let mut perception_off = 0;
    if !misc_arr[1].is_empty() 
    { 
        //NOTE: Grab initiative value
        misc_arr[0] = check_unwrap!(misc_arr[0].get(12..), file_idx, head_arr[0]);
        
        //NOTE: Look for perception at the end of senses
        //TODO: If the space after perception is missing, we can't separate it.
        let index = misc_arr[1].rfind("Percezione ");
        if index.is_some()
        {
            let index = index.unwrap();
            let senses_value = misc_arr[1].get(..index);
            if senses_value.is_none() { println!("[ERROR] Senses/Perception Fucked\n"); panic!(); }
            
            let perception_value = misc_arr[1].get(index+11..);
            if perception_value.is_none() { println!("[ERROR] Senses/Perception Fucked\n"); panic!(); }
            
            misc_arr[1] = senses_value.unwrap();
            misc_arr.insert(2, perception_value.unwrap());
            perception_off = 1;
        }
        
        senses_count = flatten_str_list(&mut misc_arr, 1, ", ");
    }
    else
    {
        //NOTE: Look for perception at the end of initiative
        let index = misc_arr[0].rfind("Percezione ");
        if index.is_some()
        {
            let index = index.unwrap();
            let init_value = misc_arr[0].get(12..index);
            
            let perception_value = misc_arr[0].get(index+11..);
            let perception_value = check_unwrap!(perception_value, file_idx, head_arr[0]);
            
            misc_arr[0] = check_unwrap!(init_value, file_idx, head_arr[0]);
            misc_arr.insert(2, perception_value);
            perception_off = 1;
        }
        else
        {
            misc_arr[0] = check_unwrap!(misc_arr[0].get(12..), file_idx, head_arr[0]);
        }
    }
    
    let defense_check   = ["PF: ", "Tiri Salvezza: ", "RD: ", "RI: ", "Immunità: ", 
                           "Resistenze: ", "Capacità Difensive: ", "Debolezze: "];
    let mut defense_arr = fill_array_from_available(&page.defense, &defense_check);
    
    let mut immunities_count = 0;
    if !defense_arr[5].is_empty()
    {
        immunities_count = flatten_str_list(&mut defense_arr, 5, ", ");
    }
    
    let mut res_count = 0;
    let res_offset = if immunities_count > 0 { immunities_count - 1 } else { 0 };
    if !defense_arr[6+res_offset].is_empty()
    {
        res_count = flatten_str_list(&mut defense_arr, 6+res_offset, ", ");
    }
    
    let mut weak_count = 0;
    let weak_offset = if res_count > 0 { (res_count - 1) + res_offset } else { res_offset };
    if !defense_arr[8+weak_offset].is_empty()
    {
        weak_count = flatten_str_list(&mut defense_arr, 8+weak_offset, ", ");
    }
    
    //NOTE: Manually fix AC
    defense_arr[0] = defense_arr[0].get(4..).unwrap();
    
    //TODO Maybe further parsing to compress and better separate attacks and spell strings?
    
    
    //NOTE: For some reason, for NPCs it's "Incantesimi" rathern than "Incantesimi Conosciuti"
    let attack_check   = ["Mischia:", "Distanza:", "Attacchi Speciali:", "Spazio:", "Portata:",
                          "Magia Psichica:", "Capacità Magiche:", "Incantesimi:" ];
    let mut attack_arr = fill_array_from_available(&page.attack, &attack_check);
    
    //NOTE: Manually fix Speed
    attack_arr[0] = attack_arr[0].get(11..).unwrap();
    
    
    //NOTE: Tactics
    let tactics_check = ["Durante il Combattimento:", "Statistiche Base:"];
    let mut tactics_arr = fill_array_from_available(&page.tactics, &tactics_check); 
    
    //NOTE: Manually fix "Prima del Combattimento"
    if !tactics_arr[0].is_empty() { tactics_arr[0] = tactics_arr[0].get(25..).unwrap(); }
    
    
    //TODO: Add checks for Dotazioni da Combattimento and Proprietà
    let stats_check   = ["Bonus di Attacco Base:", "BMC:", "DMC:", "Talenti:", "Abilità:",
                         "Linguaggi:", "Modificatori Razziali:", "Qualità Speciali:",
                         "Dotazioni da Combattimento:", "Proprietà:", "Beneficio:" ];
    let mut stats_arr = fill_array_from_available(&page.stats, &stats_check);
    
    //NOTE: Manually fix Stats
    stats_arr[0] = check_unwrap!(stats_arr[0].get(17..), file_idx, head_arr[0]);
    
    let mut talent_count = 0;
    if !stats_arr[4].is_empty()
    {
        //TODO Fix Arma Focalizzata (Spada, Arco), 
        //     Producing: [Arma Focalizzata (Spada,], [ Arco),]
        talent_count = flatten_str_list(&mut stats_arr, 4, ", ");
    }
    
    let mut skills_idx  = [0u32; 24];
    let mut skill_count = 0;
    let skill_off = if talent_count > 0 { talent_count - 1 } else { 0 };
    if !stats_arr[5+skill_off].is_empty()
    {
        //skill_count = prepare_skill_str(&mut stats_arr, &page.page_addr, skill_off);
        skills_idx = prepare_skill_str(&mut stats_arr, cache, bufs, &page.page_addr, skill_off);
    }
    
    let mut lang_count = 0;
    let lang_off = if skill_count > 0 { (skill_count - 1) + skill_off } else { skill_off };
    if !stats_arr[6+lang_off].is_empty()
    {
        lang_count = flatten_str_list(&mut stats_arr, 6+lang_off, ", ");
    }
    
    //NOTE: At the end, to keep indices correct, we unwrap all stats
    {
        let must_be_six = flatten_str_list(&mut stats_arr, 0, ", ");
        assert!(must_be_six == 6);
        
        stats_arr[0] = stats_arr[0].get(5..).unwrap().trim();
        stats_arr[1] = stats_arr[1].get(9..).unwrap().trim();
        stats_arr[2] = stats_arr[2].get(12..).unwrap().trim();
        stats_arr[3] = stats_arr[3].get(12..).unwrap().trim();
        stats_arr[4] = stats_arr[4].get(8..).unwrap().trim();
        stats_arr[5] = stats_arr[5].get(7..).unwrap().trim();
    }
    
    let mut specials_arr: Vec<&str> = Vec::new();
    let mut el: &str = "";
    let mut next: &str = &page.special;
    loop
    {
        (el, next)  = get_until(next, "\n\n");
        
        if el == GLOBAL_NULL { break; }
        
        //println!("{:#?}\n\n", el);
        specials_arr.push(el);
    }
    
    //NOTE: Manually fix source
    if page.source.len() > 7
    { page.source = check_unwrap!(page.source.get(7..), file_idx, head_arr[0]).to_string(); }
    else
    { page.source = "".to_string(); }
    
    
    // ----------------
    //NOTE Start filling the buffers
    // ----------------
    
    //Header
    let name_idx = add_entry_if_missing(&mut cache.names, &mut bufs.name, head_arr[0]);
    let gs_idx   = add_entry_if_missing(&mut cache.gs, &mut bufs.gs, head_arr[1]);
    let pe_idx   = add_entry_if_missing(&mut cache.pe, &mut bufs.pe, head_arr[2]);
    
    //Class Info
    let mut subtypes_idx = [0u16; 8];
    let mut arch_idx     = [0u16; 4];
    
    let origin_idx     = add_entry_if_missing_u32(&mut cache.strings, &mut bufs.string, &page.origin);
    let short_desc_idx = add_entry_if_missing_u32(&mut cache.strings, &mut bufs.string, class_arr[0]);
    let align_idx      = add_entry_if_missing(&mut cache.alignment, &mut bufs.alignment, class_arr[1]);
    let type_idx       = add_entry_if_missing(&mut cache.types, &mut bufs.types, class_arr[2]);
    
    for s in 0..subtypes_count
    { subtypes_idx[s]  = add_entry_if_missing(&mut cache.subtypes, &mut bufs.subtypes, class_arr[3+s]); }
    
    for arch in 0..arch_count
    { 
        let off        = 4+subtypes_off+arch;
        arch_idx[arch] = add_entry_if_missing(&mut cache.archetypes, &mut bufs.archetypes, class_arr[off]);
    }
    
    let size_idx       = add_entry_if_missing(&mut cache.sizes, &mut bufs.sizes, class_arr[5+arch_off]);
    
    //Misc
    let mut senses_idx = [0u16; 8];
    
    let init_idx       = add_entry_if_missing(&mut cache.numbers, &mut bufs.number, misc_arr[0]);
    
    for s in 0..senses_count
    { senses_idx[s]    = add_entry_if_missing(&mut cache.senses, &mut bufs.senses, misc_arr[1+s]); }
    
    let senses_off     = if senses_count > 0 { senses_count - 1 } else { 0 };
    let perception_idx = add_entry_if_missing(&mut cache.numbers, &mut bufs.number, misc_arr[2+senses_off]);
    
    let aura_off = 2+senses_off+perception_off;
    let aura_idx = add_entry_if_missing(&mut cache.auras, &mut bufs.auras, misc_arr[aura_off]);
    
    //Defense
    let mut immunities_idx  = [0u16; 16];
    let mut resistances_idx = [0u16; 16];
    let mut weaknesses_idx  = [0u16; 16];
    
    //TODO: See if I can move these into the numeric values buffer
    let ac_idx            = add_entry_if_missing_u32(&mut cache.strings, &mut bufs.string, defense_arr[0]);
    let pf_idx            = add_entry_if_missing_u32(&mut cache.strings, &mut bufs.string, defense_arr[1]);
    let st_idx            = add_entry_if_missing_u32(&mut cache.strings, &mut bufs.string, defense_arr[2]);
    let rd_idx            = add_entry_if_missing_u32(&mut cache.strings, &mut bufs.string, defense_arr[3]);
    let ri_idx            = add_entry_if_missing_u32(&mut cache.strings, &mut bufs.string, defense_arr[4]);
    
    for i in 0..immunities_count
    { immunities_idx[i]   = add_entry_if_missing(&mut cache.immunities, &mut bufs.immunities, defense_arr[5+i]); }
    
    for r in 0..res_count
    {
        let off = 6+res_offset+r;
        resistances_idx[r] = add_entry_if_missing(&mut cache.resistances, &mut bufs.resistances, defense_arr[off]);
    }
    
    let def_cap_off = 7 + weak_offset;
    let defensive_cap_idx  = add_entry_if_missing_u32(&mut cache.strings, &mut bufs.string, defense_arr[def_cap_off]);
    
    for w in 0..weak_count
    {
        let off = 8+weak_offset+w;
        weaknesses_idx[w]  = add_entry_if_missing(&mut cache.weaknesses, &mut bufs.weaknesses, defense_arr[off]);
    }
    
    //Attack
    let speed_idx    = add_entry_if_missing(&mut cache.numbers, &mut bufs.number, attack_arr[0]);
    let melee_idx    = add_entry_if_missing_u32(&mut cache.strings, &mut bufs.string, attack_arr[1]);
    let ranged_idx   = add_entry_if_missing_u32(&mut cache.strings, &mut bufs.string, attack_arr[2]);
    let spec_atk_idx = add_entry_if_missing_u32(&mut cache.strings, &mut bufs.string, attack_arr[3]);
    let space_idx    = add_entry_if_missing(&mut cache.numbers, &mut bufs.number, attack_arr[4]);
    let reach_idx    = add_entry_if_missing(&mut cache.numbers, &mut bufs.number, attack_arr[5]);
    let psych_idx    = add_entry_if_missing_u32(&mut cache.strings, &mut bufs.string, attack_arr[6]);
    let magics_idx   = add_entry_if_missing_u32(&mut cache.strings, &mut bufs.string, attack_arr[7]);
    let spells_idx   = add_entry_if_missing_u32(&mut cache.strings, &mut bufs.string, attack_arr[8]);
    
    //Tactics
    let mut tactics_idx  = [0u32; 3];
    for t in 0..tactics_idx.len()
    { tactics_idx[t] = add_entry_if_missing_u32(&mut cache.strings, &mut bufs.string, tactics_arr[t]); }
    
    //Stats
    let mut talents_idx = [0u16; 24];
    //let mut skills_idx  = [0u32; 24];
    let mut lang_idx    = [0u16; 24];
    
    let str_idx = add_entry_if_missing(&mut cache.numbers, &mut bufs.number, stats_arr[0]);
    let dex_idx = add_entry_if_missing(&mut cache.numbers, &mut bufs.number, stats_arr[1]);
    let con_idx = add_entry_if_missing(&mut cache.numbers, &mut bufs.number, stats_arr[2]);
    let int_idx = add_entry_if_missing(&mut cache.numbers, &mut bufs.number, stats_arr[3]);
    let wis_idx = add_entry_if_missing(&mut cache.numbers, &mut bufs.number, stats_arr[4]);
    let cha_idx = add_entry_if_missing(&mut cache.numbers, &mut bufs.number, stats_arr[5]);
    
    let bab_idx = add_entry_if_missing(&mut cache.numbers, &mut bufs.number, stats_arr[6]);
    let cmb_idx = add_entry_if_missing(&mut cache.numbers, &mut bufs.number, stats_arr[7]);
    let cmd_idx = add_entry_if_missing(&mut cache.numbers, &mut bufs.number, stats_arr[8]);
    
    for t in 0..talent_count
    { talents_idx[t] = add_entry_if_missing(&mut cache.talents, &mut bufs.talents, stats_arr[9+t]); }
    
    /*
    for s in 0..skill_count
    { skills_idx[s]  = add_entry_if_missing_u32(&mut cache.skills, &mut bufs.skills, stats_arr[10+s+skill_off]); }
    */
    
    //TODO: make skills cool: 25965
    // FIX Herne  Conoscenze (geografia) +6 (+8 nelle foreste)
    //println!("{:#?}:", head_arr[0]);
    /*
    for s in 0..skill_count
    { 
        let num_value_check: &[_] = &['+', '-'];
        let trim_check: &[_] = &[' ', ',', ';', '.'];
        
        if(stats_arr[10+s+skill_off] != "-")
        {
            if let Some((paren_block, paren_begin_idx, paren_end_idx)) = slice_between_inclusive(stats_arr[10+s+skill_off], "(", ")")
            {
                let mut skill_type       = [0u16; 24];
                let mut skill_value      = [-999i16; 24]; //TODO: Use -999 by default as sentinel value
                let mut skill_entry: u16 = 0;
                
                //let mut inside_paren = stats_arr[10+s+skill_off][complex_skill_idx+1..].to_lowercase();
                let mut inside_paren = paren_block.to_lowercase();
                let mut subskill_count = 0;
                for subskill_idx in 0..(Skill_Names::UnoQualsiasi as u16)
                {
                    //TODO: This might be bad. It can make me catch "+12 a saltare" and "+12 saltare"
                    //      But it might also fuck up: "tutte" vs "tutte le altre"
                    let subskill_name = &Skill_Names::from_u16(subskill_idx).as_str();
                    if(inside_paren.contains(subskill_name))
                    {
                        //NOTE: Found one
                        skill_type[subskill_count+1] = subskill_idx;
                        
                        //NOTE: Check for extra value. Is this good enough?
                        if let Some(subskill_val) = inside_paren.find(num_value_check)
                        {
                            let subskill_val_str = inside_paren[subskill_val..].split_once(' ');
                            match(subskill_val_str)
                            {
                                Some(v_str) =>
                                {
                                    skill_value[subskill_count+1] = v_str.0
                                        .trim_matches(trim_check)
                                        .parse::<i16>()
                                        .unwrap_or_else(|_| panic!("{:#?}\n[ERROR] couldn't parse value: {:#?}", head_arr[0], v_str.0));
                                    
                                    inside_paren = v_str.1.replace(subskill_name, "");
                                }
                                
                                None =>
                                {
                                    panic!("Couldn't find end of value: {:#?}", inside_paren);
                                }
                            }
                            
                        }
                        else
                        {
                            inside_paren = inside_paren.replace(subskill_name, "");
                        }
                        
                        subskill_count += 1;
                    }
                }
                
                if(subskill_count == 0)
                {
                    //println!("{:#?}:", head_arr[0]);
                    println!("\t{:#?} Interned", stats_arr[10+s+skill_off]);
                    
                    unsafe { debug_skill_cache_size += 4u32 + stats_arr[10+s+skill_off].len() as u32; }
                }
                else
                {
                    let without_paren = stats_arr[10+s+skill_off].replace(paren_block, "");
                    
                    let mut did_intern = false;
                    let mut name_check = "";
                    if let Some(v) = split_off(&without_paren, "+-")
                    {
                        name_check = v.0.trim();
                        let skill_value_opt = v.1
                            .trim_matches(trim_check)
                            .parse::<i16>();
                        
                        if(skill_value_opt.is_err())
                        {
                            println!("{:#?}:", head_arr[0]);
                            println!("[ERROR] couldn't parse value: {:#?}", stats_arr[10+s+skill_off]);
                            println!("\t{:#?} Interned", stats_arr[10+s+skill_off]);
                            
                            unsafe { debug_skill_cache_size += 4u32 + stats_arr[10+s+skill_off].len() as u32; }
                            
                            //TODO Do the interning
                            did_intern = true;
                        }
                        else
                        {
                            skill_value[0] = skill_value_opt.unwrap();
                        }
                    }
                    else
                    {
                        //NOTE: We just assume there's no bonus because it's +0...
                        name_check = without_paren.trim();
                        skill_value[0] = 0;
                    }
                    
                    if(skill_value[0] != -999)
                    {
                        skill_type[0] = Skill_Names::from_str(name_check);
                        if(skill_type[0] == 0xffff)
                        {
                            println!("\t{:#?} Interned", stats_arr[10+s+skill_off]);
                            
                            unsafe { debug_skill_cache_size += 4u32 + stats_arr[10+s+skill_off].len() as u32; }
                        }
                        else
                        {
                            skill_entry = skill_type[0] | ((skill_value[0] & 0x007F) << 7) as u16;
                            skill_entry |= 0x4000;
                            
                            //println!("\t{:#?}: {:#?} {:#?} -> {:#?}", stats_arr[10+s+skill_off], skill_type[0], skill_value[0], skill_entry);
                            
                            for subskill_idx in 0..subskill_count
                            {
                                //NOTE: All the skill entries
                                //println!("\t\t{:#?} {:#?}", Skill_Names::from_u16(skill_type[subskill_idx+1]), skill_value[subskill_idx+1]);
                            }
                        }
                    }
                    else if(did_intern == false)
                    {
                        //TODO: Probably do the interning?
                        println!("{:#?}:", head_arr[0]);
                        println!("[ERROR] couldn't parse value: {:#?}", without_paren);
                        println!("\t{:#?} Interned", stats_arr[10+s+skill_off]);
                        
                        unsafe { debug_skill_cache_size += 4u32 + stats_arr[10+s+skill_off].len() as u32; }
                    }
                }
            }
            else
            {
                let mut skill_type:  u16 = 0;
                let mut skill_value: i16 = -999;
                let mut skill_entry: u16 = 0;
                
                let mut did_intern = false;
                let mut name_check: &str = "";
                
                if let Some(v) = split_off(stats_arr[10+s+skill_off], "+-")
                {
                    //TODO: In this case there's no paren block. Which means a type without a value should be
                    //      Impossible, and if present it's an error in the source. Change this?
                    name_check = v.0.trim();
                    let skill_value_opt = v.1.trim_matches(trim_check).parse::<i16>();
                    if(skill_value_opt.is_err())
                    {
                        println!("{:#?}:", head_arr[0]);
                        println!("[ERROR] couldn't parse value: {:#?}", stats_arr[10+s+skill_off]);
                        println!("\t{:#?} Interned", stats_arr[10+s+skill_off]);
                        //TODO Do the interning
                        
                        did_intern = true;
                        
                        unsafe { debug_skill_cache_size += 4u32 + stats_arr[10+s+skill_off].len() as u32; }
                    }
                    else
                    {
                        skill_value = skill_value_opt.unwrap();
                    }
                }
                
                if(skill_value != -999)
                {
                    skill_type = Skill_Names::from_str(name_check);
                    if(skill_type == 0xffff)
                    {
                        //println!("{:#?}:", head_arr[0]);
                        //println!("Invalid Type {:#?}", stats_arr[10+s+skill_off]);
                        //TODO: Here I should Intern as Well...
                        //println!("{:#?}:", head_arr[0]);
                        //println!("[ERROR] couldn't parse value: {:#?}", stats_arr[10+s+skill_off]);
                        println!("\t{:#?} Interned", stats_arr[10+s+skill_off]);
                        
                        unsafe { debug_skill_cache_size += 4u32 + stats_arr[10+s+skill_off].len() as u32; }
                    }
                    else
                    {
                        skill_entry = skill_type | ((skill_value & 0x007F) << 7) as u16;
                        //println!("\t{:#?}: {:#?} {:#?} -> {:#?}", stats_arr[10+s+skill_off], skill_type, skill_value, skill_entry);
                    }
                }
                else if(did_intern == false)
                {
                    //TODO: Probably do the interning? Need to do did_intern boolean here as well if I do.
                    println!("{:#?}:", head_arr[0]);
                    println!("[ERROR] couldn't parse value: {:#?}", stats_arr[10+s+skill_off]);
                    println!("\t{:#?} Interned", stats_arr[10+s+skill_off]);
                    
                    unsafe { debug_skill_cache_size += 4u32 + stats_arr[10+s+skill_off].len() as u32; }
                }
            }
        }
        
        skills_idx[s] = add_entry_if_missing_u32(&mut cache.skills, &mut bufs.skills, stats_arr[10+s+skill_off]);
    }
    */
    
    for l in 0..lang_count
    { lang_idx[l]    = add_entry_if_missing(&mut cache.languages, &mut bufs.languages, stats_arr[11+l+lang_off]); }
    
    let after_lang_off = if lang_count > 0 { (lang_count - 1) + lang_off } else { lang_off };
    let racial_mods  = add_entry_if_missing_u32(&mut cache.strings, &mut bufs.string, stats_arr[12+after_lang_off]);
    let spec_qual    = add_entry_if_missing_u32(&mut cache.strings, &mut bufs.string, stats_arr[13+after_lang_off]);
    let given_equip  = add_entry_if_missing_u32(&mut cache.strings, &mut bufs.string, stats_arr[14+after_lang_off]);
    let properties   = add_entry_if_missing_u32(&mut cache.strings, &mut bufs.string, stats_arr[15+after_lang_off]);;
    let boons        = add_entry_if_missing_u32(&mut cache.strings, &mut bufs.string, stats_arr[16+after_lang_off]);;
    
    //All specials
    let mut specials_idx = [0u32; 24];
    for s in 0..specials_arr.len()
    { specials_idx[s] = add_entry_if_missing_u32(&mut cache.specials, &mut bufs.specials, &specials_arr[s]); }
    //println!("{:#?}", specials_idx);
    
    //Desc
    let desc_idx     = add_entry_if_missing_u32(&mut cache.strings, &mut bufs.string, &page.desc);
    
    //Source
    let source_idx   = add_entry_if_missing_u32(&mut cache.strings, &mut bufs.string, &page.source);
    
    let mut entry = NPC_Entry {
        name: name_idx,
        gs: gs_idx,
        pe: pe_idx,
        origin: origin_idx,
        short_desc: short_desc_idx,
        align: align_idx,
        typ: type_idx,
        sub: subtypes_idx,
        arch: arch_idx,
        size: size_idx,
        init: init_idx,
        senses: senses_idx,
        perception: perception_idx,
        aura: aura_idx,
        ac: ac_idx,
        pf: pf_idx,
        st: st_idx,
        rd: rd_idx,
        ri: ri_idx,
        immunities: immunities_idx,
        resistances: resistances_idx,
        weaknesses: weaknesses_idx,
        def_cap : defensive_cap_idx,
        speed: speed_idx,
        melee: melee_idx,
        ranged: ranged_idx,
        spec_atk: spec_atk_idx,
        space: space_idx,
        reach: reach_idx,
        psych: psych_idx,
        magics: magics_idx,
        spells: spells_idx,
        tactics: tactics_idx,
        str: str_idx,
        dex: dex_idx,
        con: con_idx,
        int: int_idx,
        wis: wis_idx,
        cha: cha_idx,
        bab: bab_idx,
        cmb: cmb_idx,
        cmd: cmd_idx,
        talents: talents_idx,
        skills: skills_idx,
        lang: lang_idx,
        racial_mods: racial_mods,
        spec_qual: spec_qual,
        given_equip: given_equip,
        properties: properties,
        boons: boons,
        specials: specials_idx,
        desc: desc_idx,
        source: source_idx,
    };
    
    return entry;
}

fn getArrayOfPaths(database_path: &str) -> Vec<String>
{
    let mut body = isahc::get(database_path).unwrap().text().unwrap();
    
    let table_offset = body.find("wiki_table_filter").unwrap();
    let table_slice  = body.get((table_offset - 11)..).unwrap();
    
    let offset  = table_slice.find("<tbody>").unwrap();
    let slice_1 = table_slice.get(offset..).unwrap();
    
    let offset_end = slice_1.find("</tbody>").unwrap();
    let slice_of_page = slice_1.get(..(offset_end+8));
    
    let base_url = "https://golarion.altervista.org";
    let mut array_of_paths = vec![];
    
    //NOTE: Skip the first <tr>
    let first_tr  = slice_of_page.unwrap().get(slice_of_page.unwrap().find("<tr>").unwrap()+1..);
    let mut curr_tr = first_tr.unwrap().get(first_tr.unwrap().find("<tr>").unwrap()+1..);
    if curr_tr.is_none() { println!("[ERROR] can't find beginning of table"); panic!(); }
    
    let (mut curr_cell, _) = get_slice_inside_tags(curr_tr.unwrap(), "<td>", "</td>");
    
    while !curr_cell.is_empty()
    {
        let mut next_slice_index = curr_cell.find("href=");
        let mut next_slice = curr_cell;
        
        while next_slice_index.is_some()
        {
            let link_slice  = next_slice.get((next_slice_index.unwrap() + 1)..);
            if link_slice.is_none() { println!("[ERROR] can't find link"); panic!(); }
            next_slice = link_slice.unwrap();
            
            let page_path  = get_path_from_slice(link_slice.unwrap());
            if page_path.is_some() {
                let s_to_push = base_url.to_string().clone() + &page_path.unwrap().to_string();
                array_of_paths.push(s_to_push);
            }
            
            next_slice_index = link_slice.unwrap().find("href=");
        }
        
        let next_tr_index = curr_tr.unwrap().find("<tr>");
        if next_tr_index.is_none() { return array_of_paths; }
        
        curr_tr = curr_tr.unwrap().get(next_tr_index.unwrap()+1..);
        (curr_cell, _) = get_slice_inside_tags(curr_tr.unwrap(),  "<td>", "</td>");
    }
    
    return array_of_paths;
}

fn get_some_pages(section_len: usize, paths: &[String], raw_pages: &mut [String])
{
    let mut slice_idx = 0;
    while slice_idx < section_len
    {
        let body = isahc::get(&paths[slice_idx]);
        if body.is_err() { continue; }
        
        let body_text = body.unwrap().text();
        if body_text.is_err() { continue; }
        
        raw_pages[slice_idx] = body_text.unwrap();
        slice_idx += 1;
    }
}

fn get_all_raw_pages(paths: &[String]) -> Vec<String>
{
    let mut raw_pages = Vec::with_capacity(paths.len());
    raw_pages.resize(paths.len(), String::new());
    
    static NUM_THREADS: usize = 32;
    
    let remainder   = paths.len() % NUM_THREADS;
    let section_len = paths.len() / NUM_THREADS;
    
    let (paths_1, paths_rest) = paths.split_at(section_len);
    let (mut raw_pages_1, mut raw_pages_rest) = raw_pages.as_mut_slice().split_at_mut(section_len);
    
    let (paths_2, paths_rest) = paths_rest.split_at(section_len);
    let (mut raw_pages_2, mut raw_pages_rest) = raw_pages_rest.split_at_mut(section_len);
    
    let (paths_3, paths_rest) = paths_rest.split_at(section_len);
    let (mut raw_pages_3, mut raw_pages_rest) = raw_pages_rest.split_at_mut(section_len);
    
    let (paths_4, paths_rest) = paths_rest.split_at(section_len);
    let (mut raw_pages_4, mut raw_pages_rest) = raw_pages_rest.split_at_mut(section_len);
    
    let (paths_5, paths_rest) = paths_rest.split_at(section_len);
    let (mut raw_pages_5, mut raw_pages_rest) = raw_pages_rest.split_at_mut(section_len);
    
    let (paths_6, paths_rest) = paths_rest.split_at(section_len);
    let (mut raw_pages_6, mut raw_pages_rest) = raw_pages_rest.split_at_mut(section_len);
    
    let (paths_7, paths_rest) = paths_rest.split_at(section_len);
    let (mut raw_pages_7, mut raw_pages_rest) = raw_pages_rest.split_at_mut(section_len);
    
    let (paths_8, paths_rest) = paths_rest.split_at(section_len);
    let (mut raw_pages_8, mut raw_pages_rest) = raw_pages_rest.split_at_mut(section_len);
    
    let (paths_9, paths_rest) = paths_rest.split_at(section_len);
    let (mut raw_pages_9, mut raw_pages_rest) = raw_pages_rest.split_at_mut(section_len);
    
    let (paths_10, paths_rest) = paths_rest.split_at(section_len);
    let (mut raw_pages_10, mut raw_pages_rest) = raw_pages_rest.split_at_mut(section_len);
    
    let (paths_11, paths_rest) = paths_rest.split_at(section_len);
    let (mut raw_pages_11, mut raw_pages_rest) = raw_pages_rest.split_at_mut(section_len);
    
    let (paths_12, paths_rest) = paths_rest.split_at(section_len);
    let (mut raw_pages_12, mut raw_pages_rest) = raw_pages_rest.split_at_mut(section_len);
    
    let (paths_13, paths_rest) = paths_rest.split_at(section_len);
    let (mut raw_pages_13, mut raw_pages_rest) = raw_pages_rest.split_at_mut(section_len);
    
    let (paths_14, paths_rest) = paths_rest.split_at(section_len);
    let (mut raw_pages_14, mut raw_pages_rest) = raw_pages_rest.split_at_mut(section_len);
    
    let (paths_15, paths_rest) = paths_rest.split_at(section_len);
    let (mut raw_pages_15, mut raw_pages_rest) = raw_pages_rest.split_at_mut(section_len);
    
    let (paths_16, paths_rest) = paths_rest.split_at(section_len);
    let (mut raw_pages_16, mut raw_pages_rest) = raw_pages_rest.split_at_mut(section_len);
    
    let (paths_17, paths_rest) = paths_rest.split_at(section_len);
    let (mut raw_pages_17, mut raw_pages_rest) = raw_pages_rest.split_at_mut(section_len);
    
    let (paths_18, paths_rest) = paths_rest.split_at(section_len);
    let (mut raw_pages_18, mut raw_pages_rest) = raw_pages_rest.split_at_mut(section_len);
    
    let (paths_19, paths_rest) = paths_rest.split_at(section_len);
    let (mut raw_pages_19, mut raw_pages_rest) = raw_pages_rest.split_at_mut(section_len);
    
    let (paths_20, paths_rest) = paths_rest.split_at(section_len);
    let (mut raw_pages_20, mut raw_pages_rest) = raw_pages_rest.split_at_mut(section_len);
    
    let (paths_21, paths_rest) = paths_rest.split_at(section_len);
    let (mut raw_pages_21, mut raw_pages_rest) = raw_pages_rest.split_at_mut(section_len);
    
    let (paths_22, paths_rest) = paths_rest.split_at(section_len);
    let (mut raw_pages_22, mut raw_pages_rest) = raw_pages_rest.split_at_mut(section_len);
    
    let (paths_23, paths_rest) = paths_rest.split_at(section_len);
    let (mut raw_pages_23, mut raw_pages_rest) = raw_pages_rest.split_at_mut(section_len);
    
    let (paths_24, paths_rest) = paths_rest.split_at(section_len);
    let (mut raw_pages_24, mut raw_pages_rest) = raw_pages_rest.split_at_mut(section_len);
    
    let (paths_25, paths_rest) = paths_rest.split_at(section_len);
    let (mut raw_pages_25, mut raw_pages_rest) = raw_pages_rest.split_at_mut(section_len);
    
    let (paths_26, paths_rest) = paths_rest.split_at(section_len);
    let (mut raw_pages_26, mut raw_pages_rest) = raw_pages_rest.split_at_mut(section_len);
    
    let (paths_27, paths_rest) = paths_rest.split_at(section_len);
    let (mut raw_pages_27, mut raw_pages_rest) = raw_pages_rest.split_at_mut(section_len);
    
    let (paths_28, paths_rest) = paths_rest.split_at(section_len);
    let (mut raw_pages_28, mut raw_pages_rest) = raw_pages_rest.split_at_mut(section_len);
    
    let (paths_29, paths_rest) = paths_rest.split_at(section_len);
    let (mut raw_pages_29, mut raw_pages_rest) = raw_pages_rest.split_at_mut(section_len);
    
    let (paths_30, paths_rest) = paths_rest.split_at(section_len);
    let (mut raw_pages_30, mut raw_pages_rest) = raw_pages_rest.split_at_mut(section_len);
    
    let (paths_31, paths_rest) = paths_rest.split_at(section_len);
    let (mut raw_pages_31, mut raw_pages_rest) = raw_pages_rest.split_at_mut(section_len);
    
    let (paths_32, paths_rest) = paths_rest.split_at(section_len);
    let (mut raw_pages_32, mut raw_pages_rest) = raw_pages_rest.split_at_mut(section_len);
    
    thread::scope(|s|
                  {
                      s.spawn(move || { get_some_pages(section_len, paths_1, raw_pages_1) });
                      s.spawn(move || { get_some_pages(section_len, paths_2, raw_pages_2) });
                      s.spawn(move || { get_some_pages(section_len, paths_3, raw_pages_3) });
                      s.spawn(move || { get_some_pages(section_len, paths_4, raw_pages_4) });
                      
                      s.spawn(move || { get_some_pages(section_len, paths_5, raw_pages_5) });
                      s.spawn(move || { get_some_pages(section_len, paths_6, raw_pages_6) });
                      s.spawn(move || { get_some_pages(section_len, paths_7, raw_pages_7) });
                      s.spawn(move || { get_some_pages(section_len, paths_8, raw_pages_8) });
                      
                      s.spawn(move || { get_some_pages(section_len, paths_9, raw_pages_9) });
                      s.spawn(move || { get_some_pages(section_len, paths_10, raw_pages_10) });
                      s.spawn(move || { get_some_pages(section_len, paths_11, raw_pages_11) });
                      s.spawn(move || { get_some_pages(section_len, paths_12, raw_pages_12) });
                      
                      s.spawn(move || { get_some_pages(section_len, paths_13, raw_pages_13) });
                      s.spawn(move || { get_some_pages(section_len, paths_14, raw_pages_14) });
                      s.spawn(move || { get_some_pages(section_len, paths_15, raw_pages_15) });
                      s.spawn(move || { get_some_pages(section_len, paths_16, raw_pages_16) });
                      
                      s.spawn(move || { get_some_pages(section_len, paths_17, raw_pages_17) });
                      s.spawn(move || { get_some_pages(section_len, paths_18, raw_pages_18) });
                      s.spawn(move || { get_some_pages(section_len, paths_19, raw_pages_19) });
                      s.spawn(move || { get_some_pages(section_len, paths_20, raw_pages_20) });
                      
                      s.spawn(move || { get_some_pages(section_len, paths_21, raw_pages_21) });
                      s.spawn(move || { get_some_pages(section_len, paths_22, raw_pages_22) });
                      s.spawn(move || { get_some_pages(section_len, paths_23, raw_pages_23) });
                      s.spawn(move || { get_some_pages(section_len, paths_24, raw_pages_24) });
                      
                      s.spawn(move || { get_some_pages(section_len, paths_25, raw_pages_25) });
                      s.spawn(move || { get_some_pages(section_len, paths_26, raw_pages_26) });
                      s.spawn(move || { get_some_pages(section_len, paths_27, raw_pages_27) });
                      s.spawn(move || { get_some_pages(section_len, paths_28, raw_pages_28) });
                      
                      s.spawn(move || { get_some_pages(section_len, paths_29, raw_pages_29) });
                      s.spawn(move || { get_some_pages(section_len, paths_30, raw_pages_30) });
                      s.spawn(move || { get_some_pages(section_len, paths_31, raw_pages_31) });
                      s.spawn(move || { get_some_pages(section_len, paths_32, raw_pages_32) });
                  });
    
    let mut file_idx = paths.len()-remainder;
    while file_idx < paths.len()
    {
        let body = isahc::get(&paths[file_idx]);
        if body.is_err() { continue; }
        
        let body_text = body.unwrap().text();
        if body_text.is_err() { continue; }
        
        raw_pages[file_idx] = body_text.unwrap();
        file_idx += 1;
    }
    
    return raw_pages;
}

//fn main() -> Result<(), Box<dyn std::error::Error>> {
fn main() -> Result<(), isahc::Error> {
    
    let now = Instant::now();
    
    let mut total_mob_get_pages_time    = 0u128;
    let mut total_npc_get_pages_time    = 0u128;
    let mut total_mob_create_entry_time = 0u128;
    let mut total_npc_create_entry_time = 0u128;
    
    let mut array_of_paths     = getArrayOfPaths("https://golarion.altervista.org/wiki/Bestiario_di_Golarion");
    let mut array_of_npc_paths = getArrayOfPaths("https://golarion.altervista.org/wiki/Database_PNG");
    
    let mut buf_context = Buffer_Context {
        string         : ByteBuffer::from_bytes(&[0u8;4]),
        number         : ByteBuffer::from_bytes(&[0u8;4]),
        name           : ByteBuffer::from_bytes(&[0u8;4]),
        gs             : ByteBuffer::from_bytes(&[0u8;4]),
        pe             : ByteBuffer::from_bytes(&[0u8;4]),
        alignment      : ByteBuffer::from_bytes(&[0u8;4]),
        types          : ByteBuffer::from_bytes(&[0u8;4]),
        subtypes       : ByteBuffer::from_bytes(&[0u8;4]),
        archetypes     : ByteBuffer::from_bytes(&[0u8;4]),
        sizes          : ByteBuffer::from_bytes(&[0u8;4]),
        senses         : ByteBuffer::from_bytes(&[0u8;4]),
        auras          : ByteBuffer::from_bytes(&[0u8;4]),
        immunities     : ByteBuffer::from_bytes(&[0u8;4]),
        resistances    : ByteBuffer::from_bytes(&[0u8;4]),
        weaknesses     : ByteBuffer::from_bytes(&[0u8;4]),
        special_attack : ByteBuffer::from_bytes(&[0u8;4]),
        spells         : ByteBuffer::from_bytes(&[0u8;4]),
        talents        : ByteBuffer::from_bytes(&[0u8;4]),
        skills         : ByteBuffer::from_bytes(&[0u8;4]),
        languages      : ByteBuffer::from_bytes(&[0u8;4]),
        specials       : ByteBuffer::from_bytes(&[0u8;4]),
        environment    : ByteBuffer::from_bytes(&[0u8;4]),
    };
    
    let mut mob_entries_vec = Vec::new();
    let mut npc_entries_vec = Vec::new();
    
    let garpm_now = Instant::now();
    let mut raw_page_vec = get_all_raw_pages(&array_of_paths);
    total_mob_get_pages_time += garpm_now.elapsed().as_millis();
    
    //let mut strings_cache = Vec::<CachedIndex>::with_capacity(4096);
    let mut vec_cache = VectorCache::new(4096);
    
    println!("Start Mobs");
    for file_idx in 0..array_of_paths.len()
        //for file_idx in 1248..1249
    {
        //println!("{file_idx} {:#?}", array_of_paths[file_idx]);
        /*
if array_of_paths[file_idx] == "https://golarion.altervista.org/wiki/Malziarax" {
            println!("{:#?}", file_idx);
        }
*/
        let mut pages = get_mob_page_array(&raw_page_vec[file_idx], &array_of_paths[file_idx]);
        
        for mut page in pages
        {
            let mce_now = Instant::now();
            let entry = create_mob_entry(&mut vec_cache, &mut buf_context, page, file_idx);
            mob_entries_vec.push(entry);
            total_mob_create_entry_time += mce_now.elapsed().as_millis();
        }
    }
    println!("End Mobs");
    
    let garpn_now = Instant::now();
    raw_page_vec = get_all_raw_pages(&array_of_npc_paths);
    total_npc_get_pages_time += garpn_now.elapsed().as_millis();
    
    println!("Start NPCs");
    for file_idx in 0..array_of_npc_paths.len()
        //for file_idx in 0..10
    {
        let mut pages = get_npc_page_array(&raw_page_vec[file_idx], &array_of_npc_paths[file_idx]);
        
        for mut page in pages
        {
            let nce_now = Instant::now();
            let entry = create_npc_entry(&mut vec_cache, &mut buf_context, page, file_idx);
            npc_entries_vec.push(entry);
            total_npc_create_entry_time += nce_now.elapsed().as_millis();
        }
    }
    println!("End NPCs");
    
    //TODO: Remove this.
    println!("Interned Skills Size: {:#?}", buf_context.skills.len());
    
    println!("Mob Count: {}", mob_entries_vec.len());
    println!("NPC Count: {}", npc_entries_vec.len());
    
    let total_size: usize = total_buffers_size(&buf_context);;
    
    println!("Total Size of Buffers: {}", total_size);
    
    dump_buffers(&buf_context);
    
    
    let mut result_file = File::create("Compendium")?;
    
    //NOTE: Write all string buffers
    result_file.write_all(&buf_context.string.to_bytes());
    result_file.write_all(&buf_context.number.to_bytes());
    result_file.write_all(&buf_context.name.to_bytes());
    result_file.write_all(&buf_context.gs.to_bytes());
    result_file.write_all(&buf_context.pe.to_bytes());
    result_file.write_all(&buf_context.alignment.to_bytes());
    result_file.write_all(&buf_context.types.to_bytes());
    result_file.write_all(&buf_context.subtypes.to_bytes());
    result_file.write_all(&buf_context.archetypes.to_bytes());
    result_file.write_all(&buf_context.sizes.to_bytes());
    result_file.write_all(&buf_context.senses.to_bytes());
    result_file.write_all(&buf_context.auras.to_bytes());
    result_file.write_all(&buf_context.immunities.to_bytes());
    result_file.write_all(&buf_context.resistances.to_bytes());
    result_file.write_all(&buf_context.weaknesses.to_bytes());
    result_file.write_all(&buf_context.special_attack.to_bytes());
    result_file.write_all(&buf_context.spells.to_bytes());
    result_file.write_all(&buf_context.talents.to_bytes());
    result_file.write_all(&buf_context.skills.to_bytes());
    result_file.write_all(&buf_context.languages.to_bytes());
    result_file.write_all(&buf_context.environment.to_bytes());
    result_file.write_all(&buf_context.specials.to_bytes());
    
    let size_before_entries = result_file.stream_position().unwrap();
    
    //NOTE: Write Mob Entries
    let mob_vec_len = mob_entries_vec.len() as u32;
    result_file.write_all([mob_vec_len].as_byte_slice());
    
    for mut entry in mob_entries_vec
    {
        result_file.write_all([entry.origin].as_byte_slice());
        result_file.write_all([entry.short_desc].as_byte_slice());
        result_file.write_all([entry.ac].as_byte_slice());
        result_file.write_all([entry.pf].as_byte_slice());
        result_file.write_all([entry.st].as_byte_slice());
        result_file.write_all([entry.rd].as_byte_slice());
        result_file.write_all([entry.ri].as_byte_slice());
        result_file.write_all([entry.def_cap].as_byte_slice());
        result_file.write_all([entry.melee].as_byte_slice());
        result_file.write_all([entry.ranged].as_byte_slice());
        result_file.write_all([entry.spec_atk].as_byte_slice());
        result_file.write_all([entry.psych].as_byte_slice());
        result_file.write_all([entry.magics].as_byte_slice());
        result_file.write_all([entry.spells].as_byte_slice());
        
        for mut el in entry.skills  { result_file.write_all([el].as_byte_slice()); }
        
        result_file.write_all([entry.racial_mods].as_byte_slice());
        result_file.write_all([entry.spec_qual].as_byte_slice());
        
        for mut el in entry.specials { result_file.write_all([el].as_byte_slice()); }
        
        result_file.write_all([entry.org].as_byte_slice());
        result_file.write_all([entry.treasure].as_byte_slice());
        result_file.write_all([entry.desc].as_byte_slice());
        result_file.write_all([entry.source].as_byte_slice());
        
        result_file.write_all([entry.name].as_byte_slice());
        result_file.write_all([entry.gs].as_byte_slice());
        result_file.write_all([entry.pe].as_byte_slice());
        result_file.write_all([entry.align].as_byte_slice());
        result_file.write_all([entry.typ].as_byte_slice());
        
        for mut el in entry.sub  { result_file.write_all([el].as_byte_slice()); }
        for mut el in entry.arch { result_file.write_all([el].as_byte_slice()); }
        
        result_file.write_all([entry.size].as_byte_slice());
        result_file.write_all([entry.init].as_byte_slice());
        
        for mut el in entry.senses { result_file.write_all([el].as_byte_slice()); }
        
        result_file.write_all([entry.perception].as_byte_slice());
        result_file.write_all([entry.aura].as_byte_slice());
        
        for mut el in entry.immunities  { result_file.write_all([el].as_byte_slice()); }
        for mut el in entry.resistances { result_file.write_all([el].as_byte_slice()); }
        for mut el in entry.weaknesses  { result_file.write_all([el].as_byte_slice()); }
        
        result_file.write_all([entry.speed].as_byte_slice());
        result_file.write_all([entry.space].as_byte_slice());
        result_file.write_all([entry.reach].as_byte_slice());
        result_file.write_all([entry.str].as_byte_slice());
        result_file.write_all([entry.dex].as_byte_slice());
        result_file.write_all([entry.con].as_byte_slice());
        result_file.write_all([entry.int].as_byte_slice());
        result_file.write_all([entry.wis].as_byte_slice());
        result_file.write_all([entry.cha].as_byte_slice());
        result_file.write_all([entry.bab].as_byte_slice());
        result_file.write_all([entry.cmb].as_byte_slice());
        result_file.write_all([entry.cmd].as_byte_slice());
        
        for mut el in entry.talents { result_file.write_all([el].as_byte_slice()); }
        for mut el in entry.lang    { result_file.write_all([el].as_byte_slice()); }
        
        result_file.write_all([entry.env].as_byte_slice());
    }
    
    let byte_padding = [0u8, 0u8];
    
    //NOTE: Write NPC Entries
    let npc_vec_len = npc_entries_vec.len() as u32;
    result_file.write_all([npc_vec_len].as_byte_slice());
    
    for mut entry in npc_entries_vec
    {
        result_file.write_all([entry.origin].as_byte_slice());
        result_file.write_all([entry.short_desc].as_byte_slice());
        result_file.write_all([entry.ac].as_byte_slice());
        result_file.write_all([entry.pf].as_byte_slice());
        result_file.write_all([entry.st].as_byte_slice());
        result_file.write_all([entry.rd].as_byte_slice());
        result_file.write_all([entry.ri].as_byte_slice());
        result_file.write_all([entry.def_cap].as_byte_slice());
        result_file.write_all([entry.melee].as_byte_slice());
        result_file.write_all([entry.ranged].as_byte_slice());
        result_file.write_all([entry.spec_atk].as_byte_slice());
        result_file.write_all([entry.psych].as_byte_slice());
        result_file.write_all([entry.magics].as_byte_slice());
        result_file.write_all([entry.spells].as_byte_slice());
        
        for mut el in entry.tactics { result_file.write_all([el].as_byte_slice()); }
        
        for mut el in entry.skills  { result_file.write_all([el].as_byte_slice()); }
        
        result_file.write_all([entry.racial_mods].as_byte_slice());
        result_file.write_all([entry.spec_qual].as_byte_slice());
        
        result_file.write_all([entry.given_equip].as_byte_slice());
        result_file.write_all([entry.properties].as_byte_slice());
        result_file.write_all([entry.boons].as_byte_slice());
        
        for mut el in entry.specials { result_file.write_all([el].as_byte_slice()); }
        
        result_file.write_all([entry.desc].as_byte_slice());
        result_file.write_all([entry.source].as_byte_slice());
        
        result_file.write_all([entry.name].as_byte_slice());
        result_file.write_all([entry.gs].as_byte_slice());
        result_file.write_all([entry.pe].as_byte_slice());
        result_file.write_all([entry.align].as_byte_slice());
        result_file.write_all([entry.typ].as_byte_slice());
        
        for mut el in entry.sub  { result_file.write_all([el].as_byte_slice()); }
        for mut el in entry.arch { result_file.write_all([el].as_byte_slice()); }
        
        result_file.write_all([entry.size].as_byte_slice());
        result_file.write_all([entry.init].as_byte_slice());
        
        for mut el in entry.senses { result_file.write_all([el].as_byte_slice()); }
        
        result_file.write_all([entry.perception].as_byte_slice());
        result_file.write_all([entry.aura].as_byte_slice());
        
        for mut el in entry.immunities  { result_file.write_all([el].as_byte_slice()); }
        for mut el in entry.resistances { result_file.write_all([el].as_byte_slice()); }
        for mut el in entry.weaknesses  { result_file.write_all([el].as_byte_slice()); }
        
        result_file.write_all([entry.speed].as_byte_slice());
        result_file.write_all([entry.space].as_byte_slice());
        result_file.write_all([entry.reach].as_byte_slice());
        result_file.write_all([entry.str].as_byte_slice());
        result_file.write_all([entry.dex].as_byte_slice());
        result_file.write_all([entry.con].as_byte_slice());
        result_file.write_all([entry.int].as_byte_slice());
        result_file.write_all([entry.wis].as_byte_slice());
        result_file.write_all([entry.cha].as_byte_slice());
        result_file.write_all([entry.bab].as_byte_slice());
        result_file.write_all([entry.cmb].as_byte_slice());
        result_file.write_all([entry.cmd].as_byte_slice());
        
        for mut el in entry.talents { result_file.write_all([el].as_byte_slice()); }
        for mut el in entry.lang    { result_file.write_all([el].as_byte_slice()); }
        
        //NOTETODO: We are adding 2 bytes of Padding because the C++ structure expects the data to be
        //          4 byte aligned, and the next closest byte alignment is 508 bytes
        //         (2 more than the current structure)
        result_file.write_all(&byte_padding);
    }
    
    let size_after_entries = result_file.stream_position().unwrap();
    
    println!("Total Bytes for Entries:         {}", size_after_entries - size_before_entries);
    println!("Total Mob Get Pages Time:        {} ms", total_mob_get_pages_time);
    println!("Total Mob Create Entity Time:    {} ms", total_mob_create_entry_time);
    
    unsafe { println!("Prepare Mob Entry Time: {} ms", create_mob_entry_prepare_time); };
    unsafe { println!("BufferMob Entry Time:   {} ms", create_mob_entry_buffer_time); };
    
    println!("Total NPC Get Pages Time:        {} ms", total_npc_get_pages_time);
    println!("Total NPC Create Entity Time:    {} ms", total_npc_create_entry_time);
    println!("Elapsed:                         {} ms", now.elapsed().as_millis());
    
    Ok(())
}
