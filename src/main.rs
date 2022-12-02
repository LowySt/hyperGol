#![allow(warnings, unused)]

use isahc::prelude::*;

use std::thread;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufWriter;
use std::io::prelude::*;
use bytebuffer::ByteBuffer;
use widestring::Utf32String;
use byte_slice_cast::*;
use byteorder::{ByteOrder, LittleEndian};

static BLACKLIST: [&str; 63] = ["/wiki/Azata", "/wiki/Agathion", "/wiki/Div", "/wiki/Drago", "/wiki/Demone", "/wiki/Daemon", "/wiki/Arconte", "/wiki/Formian", "/wiki/Demodand", "/wiki/Golem", "/wiki/Diavolo", "/wiki/Calamit%C3%A0", "/wiki/Angelo", "/wiki/Gremlin", "/wiki/Signore_dei_Demoni", "/wiki/Grande_Antico", "/wiki/Dinosauro", "/wiki/Signore_Empireo", "/wiki/Arcidiavolo", "/wiki/Linnorm", "/wiki/Behemoth", "/wiki/Sahkil", "/wiki/Oni", "/wiki/Signore_dei_Qlippoth", "/wiki/Manasaputra", "/wiki/Eone", "/wiki/Asura", "/wiki/Meccanico", "/wiki/Ombra_Notturna", "/wiki/Colosso", "/wiki/Rakshasa", "/wiki/Inevitabile", "/wiki/Caccia_Selvaggia", "/wiki/Sfinge", "/wiki/Thriae", "/wiki/Qlippoth", "/wiki/Psicopompo", "/wiki/Leshy", "/wiki/Popolo_Oscuro", "/wiki/Kami", "/wiki/Kyton", "/wiki/Protean", "/wiki/Razza_Predatrice", "/wiki/Spirito_della_Casa", "/wiki/Tsukumogami", "/wiki/Wysp", "/wiki/Carnideforme", "/wiki/Pesce", "/wiki/Robot", "/wiki/Alveare", "/wiki/Idra", "/wiki/Kaiju", "/wiki/Cavaliere_dell%27Apocalisse", "/wiki/Animale", "/wiki/Goblinoide", "/wiki/Drago_Esterno", "/wiki/Dimensione_del_Tempo", "/wiki/Razze/Munavri", "/wiki/Inferno", "/wiki/Abaddon", "/wiki/Abisso", "/wiki/Piano_Etereo", "/wiki/Elysium"];

fn check_path_against_blacklist(p: &str) -> bool {
	for v in BLACKLIST {
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
		resource.contains("Tipo") || 
		resource.contains("Sottotipo") ||
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
struct Mob_Page {
    header:  String,
    origin:  String,
    class:   String,
    misc:    String,
    defense: String,
    attack:  String,
    stats:   String,
    special: String,
    ecology: String,
    desc:    String,
    source:  String,
}

fn get_mob_page(orig_mob_page: &str) -> (Mob_Page, &str)
{
    let mut mob_page = orig_mob_page;
    
    let (mob_header, next) = get_slice_inside_tags(mob_page, "<h1>".to_string(), "</h1>".to_string());
    mob_page = next;
    
    let (race_class_info_pre, next) = get_slice_inside_tags(mob_page, "<div class=\"mw-collapsible mw-collapsed\">".to_string(), "</div>".to_string());
    mob_page = next;
    
    let (origin_info, race_class_info) = get_until(&race_class_info_pre, "<br /><i>");
    
    
    let (misc_info, next) = get_slice_inside_tags(mob_page, "<p>".to_string(), "</p>".to_string());
    mob_page = next;
    
    mob_page = skip_to(mob_page, "id=\"Difesa");
    let (defense_block, next) = get_slice_inside_tags(mob_page, "<p>".to_string(), "</p>".to_string());
    mob_page = next;
    
    mob_page = skip_to(mob_page, "id=\"Attacco");
    let (attack_block, next) = get_slice_inside_tags(mob_page, "<p>".to_string(), "<h2>".to_string());
    mob_page = next;
    
    mob_page = skip_to(mob_page, "id=\"Statistiche");
    let (stats_block, next) = get_slice_inside_tags(mob_page, "<p>".to_string(), "</p>".to_string());
    mob_page = next;
    
    let mut specials_block: &str = "";
    
    mob_page = skip_to(mob_page, "id=\"Capacità_Speciali");
    if mob_page == GLOBAL_NULL 
    {
        mob_page = skip_to(next, "id=\"Ecologia");
    }
    else
    {
        let (specials_block_tmp, next) = get_slice_inside_tags(mob_page, "<h3>".to_string(), "<h2><span class=\"mw-headline\" id=\"Ecologia".to_string());
        specials_block = specials_block_tmp;
        mob_page = next;
    }
    
    let (ecology_block, next) = get_slice_inside_tags(mob_page, "<p>".to_string(), "</p>".to_string());
    mob_page = next;
    
    mob_page = skip_to(mob_page, "id=\"Descrizione");
    let (desc_block, next) = get_slice_inside_tags(mob_page, "</h2>".to_string(), "<hr />".to_string());
    mob_page = next;
    
    let (source_block, next) = get_slice_inside_tags(mob_page, "<p>".to_string(), "</p>".to_string());
    
    // NOTE: Now we parse the sub sections.
    let mut specials = String::new();
    
    let head         = clear_all_tags(mob_header);
    let mut origin   = clear_all_tags(origin_info);
    let class        = clear_all_tags(race_class_info);
    let misc         = clear_all_tags(misc_info);
    let defense      = clear_all_tags(defense_block);
    let attack       = clear_all_tags(attack_block);
    let stats        = clear_all_tags(stats_block);
    if !specials_block.is_empty() { specials = clear_all_tags(specials_block); }
    let ecology      = clear_all_tags(ecology_block);
    let mut desc     = clear_all_tags(desc_block);
    let mut source   = clear_all_tags(source_block);
    
    if !origin.is_empty() { origin = origin.trim().to_string(); }
    if !desc.is_empty()   { desc = desc.trim().to_string(); }
    
    let mut res = Mob_Page { header: head, origin: origin, class: class, misc: misc,
        defense: defense, attack: attack, stats: stats, special: specials, 
        ecology: ecology, desc: desc, source: source };
    return (res, next.trim());
}

struct Buffer_Context
{
    string_buffer         : ByteBuffer,
    number_buffer         : ByteBuffer,
    name_buffer           : ByteBuffer,
    gs_buffer             : ByteBuffer,
    pe_buffer             : ByteBuffer,
    alignment_buffer      : ByteBuffer,
    types_buffer          : ByteBuffer,
    subtypes_buffer       : ByteBuffer,
    archetypes_buffer     : ByteBuffer,
    sizes_buffer          : ByteBuffer,
    senses_buffer         : ByteBuffer,
    auras_buffer          : ByteBuffer,
    immunities_buffer     : ByteBuffer,
    resistances_buffer    : ByteBuffer,
    weaknesses_buffer     : ByteBuffer,
    special_attack_buffer : ByteBuffer,
    spells_buffer         : ByteBuffer,
    talents_buffer        : ByteBuffer,
    skills_buffer         : ByteBuffer,
    languages_buffer      : ByteBuffer,
    specials_buffer       : ByteBuffer,
    environment_buffer    : ByteBuffer,
}


#[derive(Debug)]
struct Entry
{
    name: u16,
    gs: u16,
    pe: u16,
    origin: u32,
    short_desc: u32,
    align: u16,
    typ: u16,
    sub: [u16; 8],
    arch: [u16; 4],
    size: u16,
    init: u16,
    senses: [u16; 8],
    perception: u16,
    aura: u16,
    ac: u32,
    pf: u32,
    st: u32,
    rd: u32,
    ri: u32,
    immunities: [u16; 16],
    resistances: [u16; 16],
    weaknesses: [u16; 16],
    def_cap: u32,
    speed: u16,
    melee: u32,
    ranged: u32,
    spec_atk: u32,
    space: u16,
    reach: u16,
    psych: u32,
    magics: u32,
    spells: u32,
    str: u16,
    dex: u16,
    con: u16,
    int: u16,
    wis: u16,
    cha: u16,
    bab: u16,
    cmb: u16,
    cmd: u16,
    talents: [u16; 24],
    skills: [u16; 24],
    lang: [u16; 24],
    racial_mods: u32,
    spec_qual: u32,
    specials: [u16; 24],
    env: u16,
    org: u32,
    treasure: u32,
    desc: u32,
    source: u32,
}

fn create_entry(buf_context: &mut Buffer_Context, mut page: Mob_Page, file_idx: usize) -> Entry
{
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
        if size_idx.is_none() { println!("Maybe error? size_idx is missing. How can it be missing?"); panic!(); }
        
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
    let misc_check    = ["Sensi:", "Percezione ", "Aura:"];
    let mut misc_arr  = fill_array_from_available(&page.misc, &misc_check);
    
    //NOTE: Manually fix all misc
    let golarion_grammar_mistake = misc_arr[1].find(";"); //NOTE: Fucking golarion has grammar mistakes.
    if golarion_grammar_mistake.is_some() { misc_arr[1] = misc_arr[1].get(..misc_arr[1].len()-1).unwrap(); }
    
    misc_arr[0] = check_unwrap!(misc_arr[0].get(12..), file_idx, head_arr[0]);
    
    let mut senses_count = 0;
    if !misc_arr[1].is_empty() 
    { 
        senses_count = flatten_str_list(&mut misc_arr, 1, ", ");
    }
    
    //TODO Check auras into misc_arr
    
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
                          "Magia Psichica:", "Capacità Magiche:", "Incantesimi:" ];
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
        talent_count = flatten_str_list(&mut stats_arr, 4, ", ");
    }
    
    let mut skill_count = 0;
    let skill_off = if talent_count > 0 { talent_count - 1 } else { 0 };
    if !stats_arr[5+skill_off].is_empty()
    {
        //TODO Separate type from value and store them in 2 different buffers
        skill_count = flatten_str_list(&mut stats_arr, 5+skill_off, ", ");
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
        specials_arr.push(el);
    }
    
    let ecology_check   = ["Organizzazione:", "Tesoro:"];
    let mut ecology_arr = fill_array_from_available(&page.ecology, &ecology_check);
    
    //NOTE: Manually fix Environment
    ecology_arr[0] = check_unwrap!(ecology_arr[0].get(9..), file_idx, head_arr[0]).trim();
    
    //NOTE: Manually fix source
    page.source = check_unwrap!(page.source.get(7..), file_idx, head_arr[0]).to_string();
    
    
    // ----------------
    //NOTE Start filling the buffers
    // ----------------
    
    //Header
    let name_idx = add_entry_if_missing(&mut buf_context.name_buffer, head_arr[0]);
    let gs_idx   = add_entry_if_missing(&mut buf_context.gs_buffer, head_arr[1]);
    let pe_idx   = add_entry_if_missing(&mut buf_context.pe_buffer, head_arr[2]);
    
    //Class Info
    let mut subtypes_idx = [0u16; 8];
    let mut arch_idx     = [0u16; 4];
    
    let origin_idx     = add_entry_if_missing_u32(&mut buf_context.string_buffer, &page.origin);
    let short_desc_idx = add_entry_if_missing_u32(&mut buf_context.string_buffer, class_arr[0]);
    let align_idx      = add_entry_if_missing(&mut buf_context.alignment_buffer, class_arr[1]);
    let type_idx       = add_entry_if_missing(&mut buf_context.types_buffer, class_arr[2]);
    
    for s in 0..subtypes_count
    { subtypes_idx[s]  = add_entry_if_missing(&mut buf_context.subtypes_buffer, class_arr[3+s]); }
    
    for arch in 0..arch_count
    { arch_idx[arch]   = add_entry_if_missing(&mut buf_context.archetypes_buffer, class_arr[4+subtypes_off+arch]); }
    
    let size_idx       = add_entry_if_missing(&mut buf_context.sizes_buffer, class_arr[5+arch_off]);
    
    //Misc
    let mut senses_idx = [0u16; 8];
    
    let init_idx       = add_entry_if_missing(&mut buf_context.number_buffer, misc_arr[0]);
    
    for s in 0..senses_count
    { senses_idx[s]    = add_entry_if_missing(&mut buf_context.senses_buffer, misc_arr[1+s]); }
    
    let senses_off     = if senses_count > 0 { senses_count - 1 } else { 0 };
    let perception_idx = add_entry_if_missing(&mut buf_context.number_buffer, misc_arr[2+senses_off]);
    
    //TODO Check aura works.
    let aura_idx = add_entry_if_missing(&mut buf_context.auras_buffer, misc_arr[3+senses_off]);
    
    //Defense
    let mut immunities_idx  = [0u16; 16];
    let mut resistances_idx = [0u16; 16];
    let mut weaknesses_idx  = [0u16; 16];
    
    //TODO: See if I can move these into the numeric values buffer
    let ac_idx            = add_entry_if_missing_u32(&mut buf_context.string_buffer, defense_arr[0]);
    let pf_idx            = add_entry_if_missing_u32(&mut buf_context.string_buffer, defense_arr[1]);
    let st_idx            = add_entry_if_missing_u32(&mut buf_context.string_buffer, defense_arr[2]);
    let rd_idx            = add_entry_if_missing_u32(&mut buf_context.string_buffer, defense_arr[3]);
    let ri_idx            = add_entry_if_missing_u32(&mut buf_context.string_buffer, defense_arr[4]);
    
    for i in 0..immunities_count
    { immunities_idx[i]   = add_entry_if_missing(&mut buf_context.immunities_buffer, defense_arr[5+i]); }
    
    for r in 0..res_count
    { resistances_idx[r]  = add_entry_if_missing(&mut buf_context.resistances_buffer, defense_arr[6+res_offset+r]); }
    
    let def_cap_off = 7 + weak_offset;
    let defensive_cap_idx = add_entry_if_missing_u32(&mut buf_context.string_buffer, defense_arr[def_cap_off]);
    
    for w in 0..weak_count
    { weaknesses_idx[w]   = add_entry_if_missing(&mut buf_context.weaknesses_buffer, defense_arr[8+weak_offset+w]); }
    
    //Attack
    let speed_idx    = add_entry_if_missing(&mut buf_context.number_buffer, attack_arr[0]);
    let melee_idx    = add_entry_if_missing_u32(&mut buf_context.string_buffer, attack_arr[1]);
    let ranged_idx   = add_entry_if_missing_u32(&mut buf_context.string_buffer, attack_arr[2]);
    let spec_atk_idx = add_entry_if_missing_u32(&mut buf_context.string_buffer, attack_arr[3]);
    let space_idx    = add_entry_if_missing(&mut buf_context.number_buffer, attack_arr[4]);
    let reach_idx    = add_entry_if_missing(&mut buf_context.number_buffer, attack_arr[5]);
    let psych_idx    = add_entry_if_missing_u32(&mut buf_context.string_buffer, attack_arr[6]);
    let magics_idx   = add_entry_if_missing_u32(&mut buf_context.string_buffer, attack_arr[7]);
    let spells_idx   = add_entry_if_missing_u32(&mut buf_context.string_buffer, attack_arr[8]);
    
    //Stats
    let mut talents_idx = [0u16; 24];
    let mut skills_idx  = [0u16; 24];
    let mut lang_idx    = [0u16; 24];
    
    let str_idx = add_entry_if_missing(&mut buf_context.number_buffer, stats_arr[0]);
    let dex_idx = add_entry_if_missing(&mut buf_context.number_buffer, stats_arr[1]);
    let con_idx = add_entry_if_missing(&mut buf_context.number_buffer, stats_arr[2]);
    let int_idx = add_entry_if_missing(&mut buf_context.number_buffer, stats_arr[3]);
    let wis_idx = add_entry_if_missing(&mut buf_context.number_buffer, stats_arr[4]);
    let cha_idx = add_entry_if_missing(&mut buf_context.number_buffer, stats_arr[5]);
    
    let bab_idx = add_entry_if_missing(&mut buf_context.number_buffer, stats_arr[6]);
    let cmb_idx = add_entry_if_missing(&mut buf_context.number_buffer, stats_arr[7]);
    let cmd_idx = add_entry_if_missing(&mut buf_context.number_buffer, stats_arr[8]);
    
    for t in 0..talent_count
    { talents_idx[t] = add_entry_if_missing(&mut buf_context.talents_buffer, stats_arr[9+t]); }
    
    for s in 0..skill_count
    { skills_idx[s]  = add_entry_if_missing(&mut buf_context.skills_buffer, stats_arr[10+s+skill_off]); }
    
    for l in 0..lang_count
    { lang_idx[l]    = add_entry_if_missing(&mut buf_context.languages_buffer, stats_arr[11+l+lang_off]); }
    
    let after_lang_off = if lang_count > 0 { (lang_count - 1) + lang_off } else { lang_off };
    let racial_mods  = add_entry_if_missing_u32(&mut buf_context.string_buffer, stats_arr[12+after_lang_off]);
    let spec_qual    = add_entry_if_missing_u32(&mut buf_context.string_buffer, stats_arr[13+after_lang_off]);
    
    //All specials
    let mut specials_idx = [0u16; 24];
    for s in 0..specials_arr.len()
    { specials_idx[s] = add_entry_if_missing(&mut buf_context.specials_buffer, &specials_arr[s]); }
    
    //Ecology
    let env_idx      = add_entry_if_missing(&mut buf_context.environment_buffer, ecology_arr[0]);
    let org_idx      = add_entry_if_missing_u32(&mut buf_context.string_buffer, ecology_arr[1]);
    let treasure_idx = add_entry_if_missing_u32(&mut buf_context.string_buffer, ecology_arr[2]);
    
    //Desc
    let desc_idx     = add_entry_if_missing_u32(&mut buf_context.string_buffer, &page.desc);
    
    //Source
    let source_idx   = add_entry_if_missing_u32(&mut buf_context.string_buffer, &page.source);
    
    /*
    //if head_arr[0] == "Ragno Mannaro (Forma Umana)"
    {
        println!("IDX: {}", file_idx);
        
        for v in head_arr    { if v.is_empty() { println!("Was empty"); continue; } println!("{}", v); }
        
        if !page.origin.is_empty() { println!("{}", page.origin); }
        
        for v in class_arr   { if v.is_empty() { println!("Was empty"); continue; } println!("{}", v); }
        for v in misc_arr    { if v.is_empty() { println!("Was empty"); continue; } println!("{}", v); }
        for v in defense_arr { if v.is_empty() { println!("Was empty"); continue; } println!("{}", v); }
        for v in attack_arr  { if v.is_empty() { println!("Was empty"); continue; } println!("{}", v); }
        for v in stats_arr   { if v.is_empty() { println!("Was empty"); continue; } println!("{}", v); }
        
        if !page.special.is_empty() { println!("{}", page.special); } //NOTE: Specials is already what I want.
        
        for v in ecology_arr { if v.is_empty() { println!("Was empty"); continue; } println!("{}", v); }
        
        if page.desc.is_empty() { println!("Was empty"); } else { println!("{}", page.desc); }    //NOTE: Desc is already what I want.
        
        println!("{}", page.source);   //NOTE: Source is already what I want.
    }
    */
    
    let mut entry = Entry {
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
    
    return entry;
}

fn clear_all_tags(data_slice: &str) -> String {
    
    let mut result = String::from(data_slice);
    loop
    {
        let begin = result.find("<");
        if begin.is_none() { return result.replace("&#160;", " "); }
        
        let end = result.find(">");
        if end.is_none() { println!("[ERROR] Malformed html?"); return "".to_string(); }
        
        result.replace_range(begin.unwrap()..end.unwrap()+1, "");
    }
}

static GLOBAL_NULL: &str = "";

fn skip_to<'a>(data_slice: &'a str, tag: &str) -> &'a str
{
    let len = tag.len();
    let begin = data_slice.find(tag);
    
    if begin.is_none() { return GLOBAL_NULL; }
    
    let res = data_slice.get((begin.unwrap() + len)..);
    if res.is_none() { println!("[ERROR] Couldn't slice mob_page past tag {}", tag); return GLOBAL_NULL; }
    
    return res.unwrap();
}

fn clear_tag(data_slice: &str, tag_begin: &str, tag_end: &str) -> String {
    
    let begin_idx = data_slice.find(tag_begin);
    let end_idx   = data_slice.find(tag_end);
    
    if begin_idx.is_none() { return "".to_string(); }
    if end_idx.is_none()   { return "".to_string(); }
    
    let mut result_slice = String::from(data_slice);
    result_slice.replace_range(begin_idx.unwrap()..end_idx.unwrap(), "");
    
    return result_slice;
}

fn get_slice_inside_tags(data_slice: &str, tag_begin: String, tag_end: String) -> (&str, &str) {
    
    let begin_idx = data_slice.find(&tag_begin);
    let end_idx   = data_slice.find(&tag_end);
    
    if begin_idx.is_none() { return ("", ""); }
    if end_idx.is_none()   { return ("", ""); }
    
    let result    = data_slice.get(begin_idx.unwrap()+tag_begin.len()..end_idx.unwrap());
    if result.is_none()    { return ("", ""); }
    
    let next_data = data_slice.get(end_idx.unwrap()+tag_end.len()..);
    if next_data.is_none()    { return ("", ""); }
    
    return (result.unwrap(), next_data.unwrap());
}

fn get_until<'a>(data_slice: &'a str, until: &str) -> (&'a str, &'a str)
{
    let end_idx: usize;
    let tag_len = until.len();
    
    if until.len() == 0 { end_idx = data_slice.len(); }
    else
    {
        let index = data_slice.find(until);
        if index.is_none() { return (GLOBAL_NULL, data_slice) }
        
        end_idx = index.unwrap();
    }
    
    let result = data_slice.get(..end_idx);
    if result.is_none() { println!("[ERROR] Malformed mob page\n"); panic!(); }
    
    let tag_len = until.len();
    let next = data_slice.get(end_idx+tag_len..);
    if next.is_none() { println!("[ERROR] Malformed mob page\n"); panic!(); }
    
    return (result.unwrap(), next.unwrap());
}

fn fill_array_from_available<'a>(data_slice: &'a str, until: &[&str]) -> Vec<&'a str>
{
    let mut result_arr = vec![];
    
    let mut next: &str = data_slice;
    let mut el: &str;
    let mut i: usize = 0;
    
    let mut missed_i: usize = 99;
    
    for i in 0..until.len()
    {
        (el, next)  = get_until(next, until[i]);
        
        if el == GLOBAL_NULL && missed_i == 99 { missed_i = i; }
        
        if missed_i == 99 || el == GLOBAL_NULL { result_arr.push(el.trim()); }
        else                                   { result_arr.insert(missed_i, el.trim()); missed_i = 99; }
    }
    
    //TODO: For some reason there's an extra empty push in attack_arr...
    let (last, _) = get_until(next, "");
    if missed_i == 99 { result_arr.push(last.trim()); }
    else
    { 
        result_arr.insert(missed_i, last.trim());
        result_arr.push(GLOBAL_NULL); //Push the last missed element.
    }
    
    return result_arr;
}

//NOTETODO: Let's try with byte strings
fn add_entry_if_missing_u32(buf: &mut ByteBuffer, entry_data_str: &str) -> u32
{
    let entry_data  = entry_data_str.as_bytes();
    let entry_len   = entry_data_str.len();
    
    let mut cursor: usize = 0;
    
    //NOTE: Index 0 means an empty entry.
    if entry_len == 0 { return 0u32; }
    
    //NOTE: The first 4 bytes of the buffer represent the total length in bytes
    buf.set_rpos(4);
    while buf.get_rpos() < buf.len()
    {
        cursor         = buf.get_rpos();
        let check_size = LittleEndian::read_u16(&buf.read_bytes(2));
        let check_data = buf.read_bytes(check_size as usize);
        
        if (entry_data == check_data) == true {
            return cursor as u32;
        }
    }
    
    cursor = buf.get_wpos();
    
    buf.write_bytes([entry_len as u16].as_byte_slice());
    buf.write_bytes(entry_data);
    
    let write_cursor = buf.get_wpos();
    let new_buff_size = (buf.len() - 4) as u32;
    buf.set_wpos(0);
    buf.write_bytes([new_buff_size].as_byte_slice());
    buf.set_wpos(write_cursor);
    
    return cursor as u32;
}

fn add_entry_if_missing(buf: &mut ByteBuffer, entry_data_str: &str) -> u16
{
    let entry_data  = entry_data_str.as_bytes();
    let entry_len   = entry_data_str.len();
    
    let mut cursor: usize = 0;
    
    //NOTE: Index 0 means an empty entry.
    if entry_len == 0 { return 0u16; }
    
    //NOTE: The first 4 bytes of the buffer represent the total length in bytes
    buf.set_rpos(4);
    while buf.get_rpos() < buf.len()
    {
        cursor         = buf.get_rpos();
        let check_size = LittleEndian::read_u16(&buf.read_bytes(2));
        let check_data = buf.read_bytes(check_size as usize);
        
        if (entry_data == check_data) == true {
            return cursor as u16;
        }
    }
    
    cursor = buf.get_wpos();
    
    buf.write_bytes([entry_len as u16].as_byte_slice());
    buf.write_bytes(entry_data);
    
    let write_cursor = buf.get_wpos();
    let new_buff_size = (buf.len() - 4) as u32;
    buf.set_wpos(0);
    buf.write_bytes([new_buff_size].as_byte_slice());
    buf.set_wpos(write_cursor);
    
    return cursor as u16;
}

/* NOTETODO: Let's try with byte strings
fn add_entry(buf: &mut ByteBuffer, entry_data_str: &str) -> u16
{
    let entry_data = entry_data_str.as_bytes();
    let entry_len  = entry_data_str.len();
    let entry_utf32 = Utf32String::from_str(entry_data_str);
    
    let mut cursor: usize = 0;
    
    //NOTE: Index 0 means an empty entry.
    if entry_len == 0 { return 0u16; }
    
    
    cursor = buf.get_wpos();
    
    //NOTE we convert the string to u32 so we need the size to be 4 times larger
    let entry_bytes = entry_utf32.as_slice().len()*4;
    buf.write_u16(entry_bytes as u16);
    buf.write_bytes(entry_utf32.as_slice().as_byte_slice());
    
    let write_cursor = buf.get_wpos();
    buf.set_wpos(0);
    buf.write_u32((buf.len() - 4) as u32);
    buf.set_wpos(write_cursor);
    
    return cursor as u16;
}

fn add_entry_if_missing(buf: &mut ByteBuffer, entry_data_str: &str) -> u16
{
    let entry_data  = entry_data_str.as_bytes();
    let entry_len   = entry_data_str.len();
    let entry_utf32 = Utf32String::from_str(entry_data_str);
    
    let mut cursor: usize = 0;
    
    //NOTE: Index 0 means an empty entry.
    if entry_len == 0 { return 0u16; }
    
    //NOTE: The first 4 bytes of the buffer represent the total length in bytes
    buf.set_rpos(4);
    while buf.get_rpos() < buf.len()
    {
        cursor         = buf.get_rpos();
        let check_size = buf.read_u16();
        let check_data = buf.read_bytes(check_size as usize);
        
        if (entry_utf32.as_slice().as_byte_slice() == check_data) == true {
            return cursor as u16;
        }
    }
    
    cursor = buf.get_wpos();
    
    //NOTE we convert the string to u32 so we need the size to be 4 times larger
    let entry_bytes = entry_utf32.as_slice().len()*4;
    buf.write_u16(entry_bytes as u16);
    buf.write_bytes(entry_utf32.as_slice().as_byte_slice());
    
    let write_cursor = buf.get_wpos();
    buf.set_wpos(0);
    buf.write_u32((buf.len() - 4) as u32);
    buf.set_wpos(write_cursor);
    
    return cursor as u16;
}
*/

fn flatten_str_list(orig_arr: &mut Vec<&str>, list_idx: usize, delim: &str) -> usize
{
    let mut number_of_inserts = 0;
    let mut base = orig_arr.remove(list_idx);
    let mut idx = list_idx;
    loop
    {
        let (new_el, next) = get_until(&base, delim);
        base = next;
        
        if new_el == GLOBAL_NULL { break; }
        orig_arr.insert(idx, new_el);
        number_of_inserts += 1;
        idx += 1;
    }
    
    let (last_el, _next) = get_until(&base, "");
    orig_arr.insert(idx, last_el);
    number_of_inserts += 1;
    
    return number_of_inserts;
}

//fn main() -> Result<(), Box<dyn std::error::Error>> {
fn main() -> Result<(), isahc::Error> {
    
    use std::time::Instant;
    let now = Instant::now();
    
    //let client = reqwest::blocking::Client::new();
    //let body = client.get("https://golarion.altervista.org/wiki/Database_Mostri").send()?.text()?;
    let mut body = isahc::get("https://golarion.altervista.org/wiki/Database_Mostri")?.text()?;
    
    let offset  = body.find("wiki_table_filter").unwrap();
    let slice_1 = body.get((offset - 11)..).unwrap();
    
    let offset_end = slice_1.find("</table>").unwrap();
    let slice_of_page = slice_1.get(..(offset_end+8));
    
    let base_url = "https://golarion.altervista.org";
    let mut array_of_paths = vec![];
    
    let mut next_slice_index = slice_of_page.unwrap().find("href=");
    let mut next_slice = slice_of_page;
    while next_slice_index.is_some() && next_slice.is_some()
    {
        next_slice = next_slice.unwrap().get((next_slice_index.unwrap() + 1)..);
        if next_slice.is_none() { println!("[ERROR]"); return Ok(()); }
        
        let page_path   = get_path_from_slice(next_slice.unwrap());
        if page_path.is_some() {
            let s_to_push = base_url.to_string().clone() + &page_path.unwrap().to_string();
            array_of_paths.push(s_to_push);
        }
        
        next_slice_index = next_slice.unwrap().find("href=");
    }
    
    let mut buf_context = Buffer_Context {
        string_buffer         : ByteBuffer::from_bytes(&[0u8;4]),
        number_buffer         : ByteBuffer::from_bytes(&[0u8;4]),
        name_buffer           : ByteBuffer::from_bytes(&[0u8;4]),
        gs_buffer             : ByteBuffer::from_bytes(&[0u8;4]),
        pe_buffer             : ByteBuffer::from_bytes(&[0u8;4]),
        alignment_buffer      : ByteBuffer::from_bytes(&[0u8;4]),
        types_buffer          : ByteBuffer::from_bytes(&[0u8;4]),
        subtypes_buffer       : ByteBuffer::from_bytes(&[0u8;4]),
        archetypes_buffer     : ByteBuffer::from_bytes(&[0u8;4]),
        sizes_buffer          : ByteBuffer::from_bytes(&[0u8;4]),
        senses_buffer         : ByteBuffer::from_bytes(&[0u8;4]),
        auras_buffer          : ByteBuffer::from_bytes(&[0u8;4]),
        immunities_buffer     : ByteBuffer::from_bytes(&[0u8;4]),
        resistances_buffer    : ByteBuffer::from_bytes(&[0u8;4]),
        weaknesses_buffer     : ByteBuffer::from_bytes(&[0u8;4]),
        special_attack_buffer : ByteBuffer::from_bytes(&[0u8;4]),
        spells_buffer         : ByteBuffer::from_bytes(&[0u8;4]),
        talents_buffer        : ByteBuffer::from_bytes(&[0u8;4]),
        skills_buffer         : ByteBuffer::from_bytes(&[0u8;4]),
        languages_buffer      : ByteBuffer::from_bytes(&[0u8;4]),
        specials_buffer       : ByteBuffer::from_bytes(&[0u8;4]),
        environment_buffer    : ByteBuffer::from_bytes(&[0u8;4]),
    };
    
    let mut entries_vec = Vec::new();
    
    let mut total_size : usize = 0;
    
    for file_idx in 1800..1801
        //for file_idx in 0..array_of_paths.len()
    {
        //if((file_idx % 100) == 0) { println!("IDX: {}, {}", file_idx, array_of_paths[file_idx]); }
        println!("IDX: {}, {}", file_idx, array_of_paths[file_idx]);
        
        //thread::sleep_ms(2000);
        
        //let mob_body_opt = client.get(&array_of_paths[file_idx]).send()?.text()?;
        /*
        let mut mob_body_opt_send = reqwest::blocking::get(&array_of_paths[file_idx]);
        let maxRetries = 5;
        let mut retries    = 1;
        while(mob_body_opt_send.is_err()) //NOTE: Gonna assume is the bullshit 10054 ConnectionReset
        {
            if (retries >= maxRetries) { println!("Fuck me too many retries"); return Ok(()); }
            
            println!("Retrying after sleeping {} seconds", 2*retries);
            thread::sleep_ms(2000*retries);
            mob_body_opt_send = reqwest::blocking::get(&array_of_paths[file_idx]);
            retries += 1;
        }
        
        let mob_body_opt = mob_body_opt_send.unwrap().text()?;
        */
        
        let mut mob_body_opt = isahc::get(&array_of_paths[file_idx]);
        if mob_body_opt.is_err()
        {
            println!("Retry on error {:?}", mob_body_opt.err());
            mob_body_opt = isahc::get(&array_of_paths[file_idx]);
        }
        
        let mob_body_opt = mob_body_opt.unwrap().text().unwrap();
        
        let offset_begin = mob_body_opt.find("<h1>");
        if offset_begin.is_none() {
            println!("[ERROR] Probably non-mob page: {}", array_of_paths[file_idx]);
            return Ok(());
        }
        
        let begin_mob = mob_body_opt.get(offset_begin.unwrap()..).unwrap();
        
        let offset_end = begin_mob.find("<!--").unwrap();
        let mob_page_tmp = begin_mob.get(..offset_end);
        if mob_page_tmp.is_none() { println!("Could not do shit. Not a mob?"); return Ok(()); }
        
        let mob_page_tmp = clear_tag(mob_page_tmp.unwrap(), "<div class=\"toccolours mw-collapsible-content\">", "</div>");
        let mut mob_page = mob_page_tmp.as_str();
        
        
        let mut tmp_file = File::create("mob.html")?;
        tmp_file.write_all(mob_page.as_bytes())?;
        
        //NOTE: Let's try extracting entire tag blocks to parse the mob data
        let mut num_pages = 1;
        let (mut page_one, maybe_next) = get_mob_page(mob_page);
        
        let mut pages = Vec::new();
        pages.push(page_one);
        
        let mut page_two: Mob_Page;
        if maybe_next.len() > 3 {
            let maybe_next_tmp = clear_tag(maybe_next, "<div class=\"toccolours mw-collapsible-content\">", "</div>");
            let mut maybe_next = maybe_next_tmp.as_str();
            
            let mut tmp_file = File::create("mob_page2.html")?;
            tmp_file.write_all(maybe_next.as_bytes())?;
            
            (page_two, _) = get_mob_page(maybe_next);
            pages.push(page_two);
        }
        
        for mut page in pages
        {
            let entry = create_entry(&mut buf_context, page, file_idx);
            entries_vec.push(entry);
        }
    }
    
    total_size += buf_context.string_buffer.len();
    total_size += buf_context.number_buffer.len();
    total_size += buf_context.name_buffer.len();
    total_size += buf_context.gs_buffer.len();
    total_size += buf_context.pe_buffer.len();
    total_size += buf_context.alignment_buffer.len();
    total_size += buf_context.types_buffer.len();
    total_size += buf_context.subtypes_buffer.len();
    total_size += buf_context.archetypes_buffer.len();
    total_size += buf_context.sizes_buffer.len();
    total_size += buf_context.senses_buffer.len();
    total_size += buf_context.auras_buffer.len();
    total_size += buf_context.immunities_buffer.len();
    total_size += buf_context.resistances_buffer.len();
    total_size += buf_context.weaknesses_buffer.len();
    total_size += buf_context.special_attack_buffer.len();
    total_size += buf_context.spells_buffer.len();
    total_size += buf_context.talents_buffer.len();
    total_size += buf_context.skills_buffer.len();
    total_size += buf_context.languages_buffer.len();
    total_size += buf_context.environment_buffer.len();
    total_size += buf_context.specials_buffer.len();
    
    println!("Total Size of Buffers: {}", total_size);
    
    println!("Strings:     {}", buf_context.string_buffer.len());
    println!("Number:      {}", buf_context.number_buffer.len());
    println!("Name:        {}", buf_context.name_buffer.len());
    println!("gs:          {}", buf_context.gs_buffer.len());
    println!("pe:          {}", buf_context.pe_buffer.len());
    println!("alignment:   {}", buf_context.alignment_buffer.len());
    println!("types:       {}", buf_context.types_buffer.len());
    println!("subtypes:    {}", buf_context.subtypes_buffer.len());
    println!("archetypes:  {}", buf_context.archetypes_buffer.len());
    println!("sizes:       {}", buf_context.sizes_buffer.len());
    println!("senses:      {}", buf_context.senses_buffer.len());
    println!("auras:       {}", buf_context.auras_buffer.len());
    println!("immunities:  {}", buf_context.immunities_buffer.len());
    println!("resistances: {}", buf_context.resistances_buffer.len());
    println!("weaknesses:  {}", buf_context.weaknesses_buffer.len());
    println!("attack:      {}", buf_context.special_attack_buffer.len());
    println!("spells:      {}", buf_context.spells_buffer.len());
    println!("talents:     {}", buf_context.talents_buffer.len());
    println!("skills:      {}", buf_context.skills_buffer.len());
    println!("languages:   {}", buf_context.languages_buffer.len());
    println!("environment: {}", buf_context.environment_buffer.len());
    println!("specials:    {}", buf_context.specials_buffer.len());
    
    
    let mut result_file = File::create("Compendium")?;
    
    //NOTE: Write all string buffers
    result_file.write_all(&buf_context.string_buffer.to_bytes());
    result_file.write_all(&buf_context.number_buffer.to_bytes());
    result_file.write_all(&buf_context.name_buffer.to_bytes());
    result_file.write_all(&buf_context.gs_buffer.to_bytes());
    result_file.write_all(&buf_context.pe_buffer.to_bytes());
    result_file.write_all(&buf_context.alignment_buffer.to_bytes());
    result_file.write_all(&buf_context.types_buffer.to_bytes());
    result_file.write_all(&buf_context.subtypes_buffer.to_bytes());
    result_file.write_all(&buf_context.archetypes_buffer.to_bytes());
    result_file.write_all(&buf_context.sizes_buffer.to_bytes());
    result_file.write_all(&buf_context.senses_buffer.to_bytes());
    result_file.write_all(&buf_context.auras_buffer.to_bytes());
    result_file.write_all(&buf_context.immunities_buffer.to_bytes());
    result_file.write_all(&buf_context.resistances_buffer.to_bytes());
    result_file.write_all(&buf_context.weaknesses_buffer.to_bytes());
    result_file.write_all(&buf_context.special_attack_buffer.to_bytes());
    result_file.write_all(&buf_context.spells_buffer.to_bytes());
    result_file.write_all(&buf_context.talents_buffer.to_bytes());
    result_file.write_all(&buf_context.skills_buffer.to_bytes());
    result_file.write_all(&buf_context.languages_buffer.to_bytes());
    result_file.write_all(&buf_context.environment_buffer.to_bytes());
    result_file.write_all(&buf_context.specials_buffer.to_bytes());
    
    
    //NOTE: Write entries
    let vecLen = entries_vec.len() as u32;
    result_file.write_all([vecLen].as_byte_slice());
    
    for mut entry in entries_vec
    {
        result_file.write_all([entry.name].as_byte_slice());
        result_file.write_all([entry.gs].as_byte_slice());
        result_file.write_all([entry.pe].as_byte_slice());
        result_file.write_all([entry.origin].as_byte_slice());
        result_file.write_all([entry.short_desc].as_byte_slice());
        result_file.write_all([entry.align].as_byte_slice());
        result_file.write_all([entry.typ].as_byte_slice());
        
        for mut el in entry.sub  { result_file.write_all([el].as_byte_slice()); }
        for mut el in entry.arch { result_file.write_all([el].as_byte_slice()); }
        
        result_file.write_all([entry.size].as_byte_slice());
        result_file.write_all([entry.init].as_byte_slice());
        
        for mut el in entry.senses { result_file.write_all([el].as_byte_slice()); }
        
        result_file.write_all([entry.perception].as_byte_slice());
        result_file.write_all([entry.ac].as_byte_slice());
        result_file.write_all([entry.pf].as_byte_slice());
        result_file.write_all([entry.st].as_byte_slice());
        result_file.write_all([entry.rd].as_byte_slice());
        result_file.write_all([entry.ri].as_byte_slice());
        
        for mut el in entry.immunities  { result_file.write_all([el].as_byte_slice()); }
        for mut el in entry.resistances { result_file.write_all([el].as_byte_slice()); }
        for mut el in entry.weaknesses  { result_file.write_all([el].as_byte_slice()); }
        
        result_file.write_all([entry.def_cap].as_byte_slice());
        result_file.write_all([entry.speed].as_byte_slice());
        result_file.write_all([entry.melee].as_byte_slice());
        result_file.write_all([entry.ranged].as_byte_slice());
        result_file.write_all([entry.spec_atk].as_byte_slice());
        result_file.write_all([entry.space].as_byte_slice());
        result_file.write_all([entry.reach].as_byte_slice());
        result_file.write_all([entry.psych].as_byte_slice());
        result_file.write_all([entry.magics].as_byte_slice());
        result_file.write_all([entry.spells].as_byte_slice());
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
        for mut el in entry.skills  { result_file.write_all([el].as_byte_slice()); }
        for mut el in entry.lang    { result_file.write_all([el].as_byte_slice()); }
        
        result_file.write_all([entry.racial_mods].as_byte_slice());
        result_file.write_all([entry.spec_qual].as_byte_slice());
        
        for mut el in entry.specials { result_file.write_all([el].as_byte_slice()); }
        
        result_file.write_all([entry.env].as_byte_slice());
        result_file.write_all([entry.org].as_byte_slice());
        result_file.write_all([entry.treasure].as_byte_slice());
        result_file.write_all([entry.desc].as_byte_slice());
        result_file.write_all([entry.source].as_byte_slice());
    }
    
    
    let elapsed = now.elapsed();
    println!("Elapsed:     {:.2?}", elapsed);
    
    Ok(())
}
