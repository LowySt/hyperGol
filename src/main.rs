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
use std::hash::Hash;
use std::hash::Hasher;
use std::collections::hash_map::DefaultHasher;
use bytebuffer::ByteBuffer;
use widestring::Utf32String;
use byte_slice_cast::*;
use byteorder::{ByteOrder, LittleEndian};
/*
Current Time ~620 seconds for whole thing

For 100 Mobs + 100 NPCs
Total Mob Page Array Time:     15 seconds
    Total Single Mob Page Time:     0 seconds
  Total Get Page Time:           15 seconds
  Total Find Pieces Time:         0 seconds
Total Mob Create Entity Time:   0 seconds
Total NPC Page Array Time:     15 seconds
Total Single NPC Page Time:     0 seconds
Total NPC Create Entity Time:   0 seconds



All Mobs and NPCs with Concurrent Requests and CachedIndex on some things
Elapsed:                      78 seconds

Total Mob Get Pages Time:     27 seconds
Total Mob Create Entity Time: 10 seconds
Prepare Mob Entry Time:        0 seconds
BufferMob Entry Time:         10 seconds
Total NPC Get Pages Time:      5 seconds
Total NPC Create Entity Time: 29 seconds

Concurrent Requests and CachedIndex on all Mobs+NPCs strings
Elapsed:                      48 seconds

Total Mob Get Pages Time:     24 seconds
Total Mob Create Entity Time: 10 seconds
Prepare Mob Entry Time:        0 seconds
BufferMob Entry Time:         10 seconds
Total NPC Get Pages Time:      7 seconds
Total NPC Create Entity Time:  1 seconds

Final Time with everything on CachedIndex, Concurrenr Requests
Elapsed:                      34014 ms

Total Mob Get Pages Time:     24224 ms
Total Mob Create Entity Time:     0 ms
Prepare Mob Entry Time:           0 ms
BufferMob Entry Time:             0 ms
Total NPC Get Pages Time:      6323 ms
Total NPC Create Entity Time:     1 ms
*/

static BLACKLIST: [&str; 113] = ["/wiki/Azata", "/wiki/Agathion", "/wiki/Div", "/wiki/Drago", "/wiki/Demone", "/wiki/Daemon", "/wiki/Arconte", "/wiki/Formian", "/wiki/Demodand", "/wiki/Golem", "/wiki/Diavolo", "/wiki/Calamit%C3%A0", "/wiki/Angelo", "/wiki/Gremlin", "/wiki/Signore_dei_Demoni", "/wiki/Grande_Antico", "/wiki/Dinosauro", "/wiki/Signore_Empireo", "/wiki/Arcidiavolo", "/wiki/Linnorm", "/wiki/Behemoth", "/wiki/Sahkil", "/wiki/Oni", "/wiki/Signore_dei_Qlippoth", "/wiki/Manasaputra", "/wiki/Eone", "/wiki/Asura", "/wiki/Meccanico", "/wiki/Ombra_Notturna", "/wiki/Colosso", "/wiki/Rakshasa", "/wiki/Inevitabile", "/wiki/Caccia_Selvaggia", "/wiki/Sfinge", "/wiki/Thriae", "/wiki/Qlippoth", "/wiki/Psicopompo", "/wiki/Leshy", "/wiki/Popolo_Oscuro", "/wiki/Kami", "/wiki/Kyton", "/wiki/Protean", "/wiki/Razza_Predatrice", "/wiki/Spirito_della_Casa", "/wiki/Tsukumogami", "/wiki/Wysp", "/wiki/Carnideforme", "/wiki/Pesce", "/wiki/Robot", "/wiki/Alveare", "/wiki/Idra", "/wiki/Kaiju", "/wiki/Cavaliere_dell%27Apocalisse", "/wiki/Animale", "/wiki/Goblinoide", "/wiki/Drago_Esterno", "/wiki/Dimensione_del_Tempo", "/wiki/Razze/Munavri", "/wiki/Inferno", "/wiki/Abaddon", "/wiki/Abisso", "/wiki/Piano_Etereo", "/wiki/Elysium", "/wiki/Arcadia", "/wiki/Castrovel", "/wiki/Vudra", "/wiki/Piaga_del_Mondo", "/wiki/Korvosa", "/wiki/Cheliax", "/wiki/Rahadoum", "/wiki/Garund", "/wiki/Paradiso", "/wiki/Kaer_Maga", "/wiki/Desolazioni_del_Mana", "/wiki/Ossario", "/wiki/Axis", "/wiki/Nuat", "/wiki/Osirion", "/wiki/Lande_Tenebrose", "/wiki/Piano_delle_Ombre", "/wiki/Fiume_Stige", "/wiki/Campo_delle_Fanciulle", "/wiki/Razmiran", "/wiki/Deserto_Piagamagica", "/wiki/Nirvana", "/wiki/Varisia", "/wiki/Katapesh", "/wiki/Distese_Mwangi", "/wiki/Piano_dell%27Energia_Negativa", "/wiki/Abaddon", "/wiki/Isola_Mediogalti", "/wiki/Piano_Elementale_della_Terra", "/wiki/Piano_Elementale_della_Terra", "/wiki/Dimensione_del_Tempo", "/wiki/Occhio_di_Abendego", "/wiki/Lande_Cineree", "/wiki/Crystilan", "/wiki/Xin-Edasseril", "/wiki/Numeria", "/wiki/Thassilon", "/wiki/Kalexcourt", "/wiki/Ustalav", "/wiki/Quantium", "/wiki/Casmaron", "/wiki/Foresta_Grungir", "/wiki/Piano_Materiale", "/wiki/Siktempora", "/wiki/Araldo", "/wiki/Progenie_di_Rovagug", "/wiki/Kyton#Kyton_Demagogo", "/wiki/Limbo", "/wiki/Piano_Elementale_dell%27Acqua", "/wiki/Piano_Elementale_dell%27Aria"];

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

//TODO: Merge this with mob page in the future
#[derive(Debug)]
struct NPC_Page {
    header:  String,
    origin:  String,
    class:   String,
    misc:    String,
    defense: String,
    attack:  String,
    tactics: String,
    stats:   String,
    special: String,
    desc:    String,
    source:  String,
}

fn get_npc_page(orig_npc_page: &str) -> NPC_Page
{
    let mut npc_page = orig_npc_page;
    
    let (npc_header, next) = get_slice_inside_tags(npc_page, "<h1>".to_string(), "</h1>".to_string());
    npc_page = next;
    
    let (race_class_info_pre, next) = get_slice_inside_tags(npc_page, "<div class=\"mw-collapsible mw-collapsed\">".to_string(), "</div>".to_string());
    npc_page = next;
    
    let (mut origin_info, mut race_class_info) = get_until(&race_class_info_pre, "<br /><i>");
    
    //NOTE: This means there is no description, but we still don't know if there is no origin
    //      We need to check for Allineamento and see how far away it is from race_class_info_pre.
    //      If it is far enough away (more than 3 characters of a tag) we have an origin, otherwise
    //      origin stays empty.
    if origin_info == GLOBAL_NULL
    {
        let mut alignIndex = race_class_info_pre.find("<b>Allineamento").unwrap();
        
        //NOTE: Means we are far away from the beginning of race_class_info_pre
        //      which means there has to be an origin
        if alignIndex > 3
        {
            origin_info     = race_class_info_pre.get(..alignIndex).unwrap();
            race_class_info = race_class_info_pre.get(alignIndex..).unwrap();
        }
        else //NOTE: There is no origin, only race_class_info
        {
            race_class_info = race_class_info_pre;
        }
    }
    
    let (misc_info, next) = get_slice_inside_tags(npc_page, "<p>".to_string(), "</p>".to_string());
    npc_page = next;
    
    npc_page = skip_to(npc_page, "id=\"Difesa");
    let (defense_block, next) = get_slice_inside_tags(npc_page, "<p>".to_string(), "</p>".to_string());
    npc_page = next;
    
    npc_page = skip_to(npc_page, "id=\"Attacco");
    let (attack_block, next) = get_slice_inside_tags(npc_page, "<p>".to_string(), "<h2>".to_string());
    npc_page = next;
    
    let mut tactics_block: &str = "";
    npc_page = skip_to(npc_page, "id=\"Tattiche");
    if npc_page != GLOBAL_NULL
    {
        let (tactics_block_tmp, next) = get_slice_inside_tags(npc_page, "<p>".to_string(), "<h2>".to_string());
        tactics_block = tactics_block_tmp;
        npc_page = next;
    }
    else { npc_page = next; }
    
    npc_page = skip_to(npc_page, "id=\"Statistiche");
    let (stats_block, next) = get_slice_inside_tags(npc_page, "<p>".to_string(), "</p>".to_string());
    npc_page = next;
    
    let mut specials_block: &str = "";
    npc_page = skip_to(npc_page, "id=\"Capacità_Speciali");
    if npc_page != GLOBAL_NULL
    {
        let (specials_block_tmp, next) = get_slice_inside_tags(npc_page, "<h3>".to_string(), "<h2><span class=\"mw-headline\" id=\"Descrizione".to_string());
        specials_block = specials_block_tmp;
        npc_page = next;
    }
    else { npc_page = next; }
    
    let mut desc_block: &str = "";
    npc_page = skip_to(npc_page, "id=\"Descrizione");
    if npc_page != GLOBAL_NULL
    {
        let (desc_block_tmp, next) = get_slice_inside_tags(npc_page, "</h2>".to_string(), "<hr />".to_string());
        desc_block = desc_block_tmp;
        npc_page = next;
    }
    else { npc_page = next; }
    
    npc_page = skip_to(npc_page, "<p>Fonte:");
    //let (source_block, next) = get_slice_inside_tags(npc_page, "<p>".to_string(), "</p>".to_string());
    let (source_block, next) = get_until(npc_page, "</p>");
    
    // NOTE: Now we parse the sub sections.
    let mut specials = String::new();
    
    let head         = clear_all_tags(npc_header);
    let mut origin   = clear_all_tags(origin_info);
    let class        = clear_all_tags(race_class_info);
    let misc         = clear_all_tags(misc_info);
    let defense      = clear_all_tags(defense_block);
    let attack       = clear_all_tags(attack_block);
    let tactics      = clear_all_tags(tactics_block);
    let stats        = clear_all_tags(stats_block);
    if !specials_block.is_empty() { specials = clear_all_tags(specials_block); }
    let mut desc     = clear_all_tags(desc_block);
    let mut source   = clear_all_tags(source_block);
    
    if !origin.is_empty() { origin = origin.trim().to_string(); }
    if !desc.is_empty()   { desc = desc.trim().to_string(); }
    
    let mut res = NPC_Page { header: head, origin: origin, class: class, misc: misc,
        defense: defense, attack: attack, tactics: tactics, 
        stats: stats, special: specials, desc: desc, source: source };
    
    return res;
}

struct Buffer_Context
{
    string         : ByteBuffer,
    number         : ByteBuffer,
    name           : ByteBuffer,
    gs             : ByteBuffer,
    pe             : ByteBuffer,
    alignment      : ByteBuffer,
    types          : ByteBuffer,
    subtypes       : ByteBuffer,
    archetypes     : ByteBuffer,
    sizes          : ByteBuffer,
    senses         : ByteBuffer,
    auras          : ByteBuffer,
    immunities     : ByteBuffer,
    resistances    : ByteBuffer,
    weaknesses     : ByteBuffer,
    special_attack : ByteBuffer,
    spells         : ByteBuffer,
    talents        : ByteBuffer,
    skills         : ByteBuffer,
    languages      : ByteBuffer,
    specials       : ByteBuffer,
    environment    : ByteBuffer,
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
    skills     : [u16; 24],
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
    let mut skills_idx  = [0u16; 24];
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
    
    for s in 0..skill_count
    { skills_idx[s]  = add_entry_if_missing(&mut cache.skills, &mut bufs.skills, stats_arr[10+s+skill_off]); }
    
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
    tactics    : u32,
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
    skills     : [u16; 24],
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
    let attack_check   = ["Mischia:", "Distanza:", "Attacchi Speciali:", "Spazio:", "Portata:",
                          "Magia Psichica:", "Capacità Magiche:", "Incantesimi Conosciuti:" ];
    let mut attack_arr = fill_array_from_available(&page.attack, &attack_check);
    
    
    //NOTE: Tactics
    //TODO: I don't know, parse it more?
    //page.tactics = check_unwrap!(page.tactics, file_idx, head_arr[0]).to_string();
    
    //NOTE: Manually fix Speed
    attack_arr[0] = attack_arr[0].get(11..).unwrap();
    
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
    let tactics_idx  = add_entry_if_missing_u32(&mut cache.strings, &mut bufs.string, &page.tactics);
    
    //Stats
    let mut talents_idx = [0u16; 24];
    let mut skills_idx  = [0u16; 24];
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
    
    for s in 0..skill_count
    { skills_idx[s]  = add_entry_if_missing(&mut cache.skills, &mut bufs.skills, stats_arr[10+s+skill_off]); }
    
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

fn clear_all_tags(data_slice: &str) -> String {
    
    let mut result = String::from(data_slice);
    loop
    {
        let begin = result.find("<");
        if begin.is_none() { return result.replace("&#160;", " "); }
        
        let end = result.find(">");
        if end.is_none() { println!("[ERROR] Malformed html?"); return "".to_string(); }
        
        if begin.unwrap() > end.unwrap() { println!("[ERROR] Bad assumption?"); return "".to_string(); }
        
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
        (el, next) = get_until(next, until[i]);
        
        if el == GLOBAL_NULL && missed_i == 99 { missed_i = i; }
        
        if missed_i == 99 || el == GLOBAL_NULL { result_arr.push(el.trim()); }
        else                                   { result_arr.insert(missed_i, el.trim()); missed_i = 99; }
    }
    
    let (last, _) = get_until(next, "");
    if missed_i == 99 { result_arr.push(last.trim()); }
    else
    { 
        result_arr.insert(missed_i, last.trim());
        result_arr.push(GLOBAL_NULL); //Push the last missed element.
    }
    
    return result_arr;
}

struct CachedIndex<T>
{
    hash: u64,
    cursor: T,
}

struct VectorCache
{
    strings:     Vec::<CachedIndex::<u32>>,
    numbers:     Vec::<CachedIndex::<u16>>,
    names:       Vec::<CachedIndex::<u16>>, //TODO We are gonna surpass 16bits veery soon!
    gs:          Vec::<CachedIndex::<u16>>,
    pe:          Vec::<CachedIndex::<u16>>,
    alignment:   Vec::<CachedIndex::<u16>>,
    types:       Vec::<CachedIndex::<u16>>,
    subtypes:    Vec::<CachedIndex::<u16>>,
    archetypes:  Vec::<CachedIndex::<u16>>,
    sizes:       Vec::<CachedIndex::<u16>>,
    senses:      Vec::<CachedIndex::<u16>>,
    auras:       Vec::<CachedIndex::<u16>>,
    immunities:  Vec::<CachedIndex::<u16>>,
    resistances: Vec::<CachedIndex::<u16>>,
    weaknesses:  Vec::<CachedIndex::<u16>>,
    special:     Vec::<CachedIndex::<u16>>,
    spells:      Vec::<CachedIndex::<u32>>,
    talents:     Vec::<CachedIndex::<u16>>,
    skills:      Vec::<CachedIndex::<u16>>,
    languages:   Vec::<CachedIndex::<u16>>,
    specials:    Vec::<CachedIndex::<u32>>,
    environment: Vec::<CachedIndex::<u16>>,
}

impl VectorCache
{
    fn new(pre_alloc: usize) -> VectorCache
    {
        let result = VectorCache {
            strings:     Vec::<CachedIndex::<u32>>::with_capacity(pre_alloc),
            numbers:     Vec::<CachedIndex::<u16>>::with_capacity(pre_alloc),
            names:       Vec::<CachedIndex::<u16>>::with_capacity(pre_alloc),
            gs:          Vec::<CachedIndex::<u16>>::with_capacity(pre_alloc),
            pe:          Vec::<CachedIndex::<u16>>::with_capacity(pre_alloc),
            alignment:   Vec::<CachedIndex::<u16>>::with_capacity(pre_alloc),
            types:       Vec::<CachedIndex::<u16>>::with_capacity(pre_alloc),
            subtypes:    Vec::<CachedIndex::<u16>>::with_capacity(pre_alloc),
            archetypes:  Vec::<CachedIndex::<u16>>::with_capacity(pre_alloc),
            sizes:       Vec::<CachedIndex::<u16>>::with_capacity(pre_alloc),
            senses:      Vec::<CachedIndex::<u16>>::with_capacity(pre_alloc),
            auras:       Vec::<CachedIndex::<u16>>::with_capacity(pre_alloc),
            immunities:  Vec::<CachedIndex::<u16>>::with_capacity(pre_alloc),
            resistances: Vec::<CachedIndex::<u16>>::with_capacity(pre_alloc),
            weaknesses:  Vec::<CachedIndex::<u16>>::with_capacity(pre_alloc),
            special:     Vec::<CachedIndex::<u16>>::with_capacity(pre_alloc),
            spells:      Vec::<CachedIndex::<u32>>::with_capacity(pre_alloc),
            talents:     Vec::<CachedIndex::<u16>>::with_capacity(pre_alloc),
            skills:      Vec::<CachedIndex::<u16>>::with_capacity(pre_alloc),
            languages:   Vec::<CachedIndex::<u16>>::with_capacity(pre_alloc),
            specials:    Vec::<CachedIndex::<u32>>::with_capacity(pre_alloc),
            environment: Vec::<CachedIndex::<u16>>::with_capacity(pre_alloc),
        };
        return result;
    }
}

fn add_entry_if_missing_u32(cache: &mut Vec<CachedIndex<u32>>, buf: &mut ByteBuffer, entry_data_str: &str) -> u32
{
    let replace_shit = entry_data_str.replace("\u{2012}", "-");
    let replace_shit = replace_shit.replace("\u{200b}", "");
    let replace_shit = replace_shit.replace("\u{2011}", "-");
    let replace_shit = replace_shit.replace("\u{2013}", "-");
    let replace_shit = replace_shit.replace("\u{2014}", "-");
    let replace_shit = replace_shit.replace("\u{2018}", "'");
    let replace_shit = replace_shit.replace("\u{2019}", "'");
    let replace_shit = replace_shit.replace("\u{201c}", "\"");
    let replace_shit = replace_shit.replace("\u{201d}", "\"");
    let replace_shit = replace_shit.replace("\u{2026}", "...");
    let replace_shit = replace_shit.replace("\u{2800}", "");
    let replace_shit = replace_shit.replace("\u{fb01}", "fi");
    let replace_shit = replace_shit.replace("\u{fb02}", "fl");
    let entry_data   = replace_shit.as_bytes();
    let entry_len    = replace_shit.len();
    
    //NOTE: Index 0 means an empty entry.
    if entry_len == 0 { return 0u32; }
    
    let mut hasher = DefaultHasher::new();
    entry_data.hash(&mut hasher);
    let hash = hasher.finish();
    
    for idx in 0..cache.len()
    { 
        if cache[idx].hash == hash { return cache[idx].cursor; }
    }
    
    let mut cursor: usize = 0;
    cursor = buf.get_wpos();
    
    buf.write_bytes([entry_len as u16].as_byte_slice());
    buf.write_bytes(entry_data);
    
    let write_cursor = buf.get_wpos();
    let new_buff_size = (buf.len() - 4) as u32;
    buf.set_wpos(0);
    buf.write_bytes([new_buff_size].as_byte_slice());
    buf.set_wpos(write_cursor);
    
    let mut cached_index = CachedIndex { hash: hash, cursor: cursor as u32 };
    cache.push(cached_index);
    
    return cursor as u32;
}
fn add_entry_if_missing(cache: &mut Vec<CachedIndex<u16>>, buf: &mut ByteBuffer, entry_data_str: &str) -> u16
{
    let replace_shit = str::replace(entry_data_str, "\u{2012}", "-");
    let replace_shit = replace_shit.replace("\u{200b}", "");
    let replace_shit = replace_shit.replace("\u{2011}", "-");
    let replace_shit = replace_shit.replace("\u{2013}", "-");
    let replace_shit = replace_shit.replace("\u{2014}", "-");
    let replace_shit = replace_shit.replace("\u{2018}", "'");
    let replace_shit = replace_shit.replace("\u{2019}", "'");
    let replace_shit = replace_shit.replace("\u{201c}", "\"");
    let replace_shit = replace_shit.replace("\u{201d}", "\"");
    let replace_shit = replace_shit.replace("\u{2026}", "...");
    let replace_shit = replace_shit.replace("\u{2800}", "");
    let replace_shit = replace_shit.replace("\u{fb01}", "fi");
    let replace_shit = replace_shit.replace("\u{fb02}", "fl");
    let entry_data   = replace_shit.as_bytes();
    let entry_len    = replace_shit.len();
    
    //NOTE: Index 0 means an empty entry.
    if entry_len == 0 { return 0u16; }
    
    let mut hasher = DefaultHasher::new();
    entry_data.hash(&mut hasher);
    let hash = hasher.finish();
    
    for idx in 0..cache.len()
    { 
        if cache[idx].hash == hash { return cache[idx].cursor; }
    }
    
    let mut cursor: usize = 0;
    cursor = buf.get_wpos();
    
    buf.write_bytes([entry_len as u16].as_byte_slice());
    buf.write_bytes(entry_data);
    
    let write_cursor = buf.get_wpos();
    let new_buff_size = (buf.len() - 4) as u32;
    buf.set_wpos(0);
    buf.write_bytes([new_buff_size].as_byte_slice());
    buf.set_wpos(write_cursor);
    
    let mut cached_index = CachedIndex { hash: hash, cursor: cursor as u16 };
    cache.push(cached_index);
    
    return cursor as u16;
}

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

fn get_mob_page_array(mob_body_opt: &str, page_path: &str) -> Vec<Mob_Page>
{
    let offset_begin = mob_body_opt.find("<h1>");
    if offset_begin.is_none() {
        println!("[ERROR] Probably non-mob page: {}", page_path);
        panic!();
    }
    
    let begin_mob = mob_body_opt.get(offset_begin.unwrap()..).unwrap();
    
    let offset_end = begin_mob.find("<!--").unwrap();
    let mob_page_tmp = begin_mob.get(..offset_end);
    if mob_page_tmp.is_none() { println!("Could not do shit. Not a mob?"); panic!(); }
    
    let mob_page_tmp = clear_tag(mob_page_tmp.unwrap(), "<div class=\"toccolours mw-collapsible-content\"", "</div>");
    
    let mut mob_page = mob_page_tmp.as_str();
    
    //NOTE: Let's try extracting entire tag blocks to parse the mob data
    let mut num_pages = 1;
    let (mut page_one, maybe_next) = get_mob_page(mob_page);
    
    let mut pages = Vec::new();
    pages.push(page_one);
    
    let mut page_two: Mob_Page;
    if maybe_next.len() > 3 {
        let maybe_next_tmp = clear_tag(maybe_next, "<div class=\"toccolours mw-collapsible-content\"", "</div>");
        let mut maybe_next = maybe_next_tmp.as_str();
        
        (page_two, _) = get_mob_page(maybe_next);
        pages.push(page_two);
    }
    
    return pages;
}

fn get_npc_page_array(mob_body_opt: &str, page_path: &str) -> Vec<NPC_Page>
{
    let mut has_two_pages = false;
    /*
    let mut mob_body_opt = isahc::get(page_path);
    if mob_body_opt.is_err()
    {
        println!("Retry on error {:?}", mob_body_opt.err());
        mob_body_opt = isahc::get(page_path);
    }
    
    let mob_body_opt = mob_body_opt.unwrap().text().unwrap();
    */
    
    let offset_begin = mob_body_opt.find("<h1>");
    if offset_begin.is_none() {
        println!("[ERROR] Probably non-npc page: {}", page_path);
        panic!();
    }
    
    let second_begin = mob_body_opt.rfind("<h1>");
    if second_begin.is_some() && second_begin.unwrap() > offset_begin.unwrap() {
        has_two_pages = true;
    }
    
    let mut mob_page = String::new();
    let mut second_page = String::new();
    if has_two_pages
    {
        let mob_page_tmp = mob_body_opt.get(offset_begin.unwrap()..second_begin.unwrap());
        if mob_page_tmp.is_none() { println!("Could not do shit in first page. Not a npc?"); panic!(); }
        
        mob_page = clear_tag(mob_page_tmp.unwrap(), "<div class=\"toccolours mw-collapsible-content\"", "</div>");
        
        let second_page_begin_tmp = mob_body_opt.get(second_begin.unwrap()..).unwrap();
        let second_end = second_page_begin_tmp.find("<!--").unwrap();
        
        let second_page_tmp = second_page_begin_tmp.get(..second_end);
        if second_page_tmp.is_none() { println!("Second page didn't work."); panic!(); }
        
        second_page = clear_tag(second_page_tmp.unwrap(), "<div class=\"toccolours mw-collapsible-content\"", "</div>");
    }
    else
    {
        let begin_mob = mob_body_opt.get(offset_begin.unwrap()..).unwrap();
        let offset_end = begin_mob.find("<!--").unwrap();
        
        let mob_page_tmp = begin_mob.get(..offset_end);
        if mob_page_tmp.is_none() { println!("Could not do shit in first page. Not a npc?"); panic!(); }
        
        mob_page = clear_tag(mob_page_tmp.unwrap(), "<div class=\"toccolours mw-collapsible-content\"", "</div>");
    }
    
    let page_one = get_npc_page(&mob_page);
    
    let mut pages = Vec::new();
    pages.push(page_one);
    
    if has_two_pages
    {
        let page_two = get_npc_page(&second_page);
        pages.push(page_two);
    }
    
    return pages;
}

fn getArrayOfPaths(database_path: &str) -> Vec<String>
{
    let mut body = isahc::get(database_path).unwrap().text().unwrap();
    
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
        if next_slice.is_none() { println!("[ERROR]"); panic!(); }
        
        let page_path   = get_path_from_slice(next_slice.unwrap());
        if page_path.is_some() {
            let s_to_push = base_url.to_string().clone() + &page_path.unwrap().to_string();
            array_of_paths.push(s_to_push);
        }
        
        next_slice_index = next_slice.unwrap().find("href=");
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
    {
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
    
    let mut total_size : usize = 0;
    
    total_size += buf_context.string.len();
    total_size += buf_context.number.len();
    total_size += buf_context.name.len();
    total_size += buf_context.gs.len();
    total_size += buf_context.pe.len();
    total_size += buf_context.alignment.len();
    total_size += buf_context.types.len();
    total_size += buf_context.subtypes.len();
    total_size += buf_context.archetypes.len();
    total_size += buf_context.sizes.len();
    total_size += buf_context.senses.len();
    total_size += buf_context.auras.len();
    total_size += buf_context.immunities.len();
    total_size += buf_context.resistances.len();
    total_size += buf_context.weaknesses.len();
    total_size += buf_context.special_attack.len();
    total_size += buf_context.spells.len();
    total_size += buf_context.talents.len();
    total_size += buf_context.skills.len();
    total_size += buf_context.languages.len();
    total_size += buf_context.environment.len();
    total_size += buf_context.specials.len();
    
    println!("Total Size of Buffers: {}", total_size);
    
    println!("Strings:     {}", buf_context.string.len());
    println!("Number:      {}", buf_context.number.len());
    println!("Name:        {}", buf_context.name.len());
    println!("gs:          {}", buf_context.gs.len());
    println!("pe:          {}", buf_context.pe.len());
    println!("alignment:   {}", buf_context.alignment.len());
    println!("types:       {}", buf_context.types.len());
    println!("subtypes:    {}", buf_context.subtypes.len());
    println!("archetypes:  {}", buf_context.archetypes.len());
    println!("sizes:       {}", buf_context.sizes.len());
    println!("senses:      {}", buf_context.senses.len());
    println!("auras:       {}", buf_context.auras.len());
    println!("immunities:  {}", buf_context.immunities.len());
    println!("resistances: {}", buf_context.resistances.len());
    println!("weaknesses:  {}", buf_context.weaknesses.len());
    println!("attack:      {}", buf_context.special_attack.len());
    println!("spells:      {}", buf_context.spells.len());
    println!("talents:     {}", buf_context.talents.len());
    println!("skills:      {}", buf_context.skills.len());
    println!("languages:   {}", buf_context.languages.len());
    println!("environment: {}", buf_context.environment.len());
    println!("specials:    {}", buf_context.specials.len());
    
    
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
        for mut el in entry.skills  { result_file.write_all([el].as_byte_slice()); }
        for mut el in entry.lang    { result_file.write_all([el].as_byte_slice()); }
        
        result_file.write_all([entry.env].as_byte_slice());
    }
    
    let byte_padding = [0u8, 0u8];
    
    //NOTE: Write NpC Entries
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
        
        result_file.write_all([entry.tactics].as_byte_slice());
        
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
        for mut el in entry.skills  { result_file.write_all([el].as_byte_slice()); }
        for mut el in entry.lang    { result_file.write_all([el].as_byte_slice()); }
        
        //NOTETODO: We are adding 2 bytes of Padding because the C++ structure expects the data to be
        //          4 byte aligned, and the next closest byte alignment is 508 bytes
        //         (2 more than the current structure)
        result_file.write_all(&byte_padding);
    }
    
    
    println!("Total Mob Get Pages Time:     {} ms", total_mob_get_pages_time);
    println!("Total Mob Create Entity Time: {} ms", total_mob_create_entry_time);
    
    unsafe { println!("Prepare Mob Entry Time: {} ms", create_mob_entry_prepare_time); };
    unsafe { println!("BufferMob Entry Time:   {} ms", create_mob_entry_buffer_time); };
    
    println!("Total NPC Get Pages Time:     {} ms", total_npc_get_pages_time);
    println!("Total NPC Create Entity Time: {} ms", total_npc_create_entry_time);
    println!("Elapsed:                      {} ms", now.elapsed().as_millis());
    
    Ok(())
}
