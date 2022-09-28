#![allow(warnings, unused)]

use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufWriter;
use std::io::prelude::*;
use bytebuffer::ByteBuffer;

static BLACKLIST: [&str; 58] = ["/wiki/Azata", "/wiki/Agathion", "/wiki/Div", "/wiki/Drago", "/wiki/Demone", "/wiki/Daemon", "/wiki/Arconte", "/wiki/Formian", "/wiki/Demodand", "/wiki/Golem", "/wiki/Diavolo", "/wiki/Calamit%C3%A0", "/wiki/Angelo", "/wiki/Gremlin", "/wiki/Signore_dei_Demoni", "/wiki/Grande_Antico", "/wiki/Dinosauro", "/wiki/Signore_Empireo", "/wiki/Arcidiavolo", "/wiki/Linnorm", "/wiki/Behemoth", "/wiki/Sahkil", "/wiki/Oni", "/wiki/Signore_dei_Qlippoth", "/wiki/Manasaputra", "/wiki/Eone", "/wiki/Asura", "/wiki/Meccanico", "/wiki/Ombra_Notturna", "/wiki/Colosso", "/wiki/Rakshasa", "/wiki/Inevitabile", "/wiki/Caccia_Selvaggia", "/wiki/Sfinge", "/wiki/Thriae", "/wiki/Qlippoth", "/wiki/Psicopompo", "/wiki/Leshy", "/wiki/Popolo_Oscuro", "/wiki/Kami", "/wiki/Kyton", "/wiki/Protean", "/wiki/Razza_Predatrice", "/wiki/Spirito_della_Casa", "/wiki/Tsukumogami", "/wiki/Wysp", "/wiki/Carnideforme", "/wiki/Pesce", "/wiki/Robot", "/wiki/Alveare", "/wiki/Idra", "/wiki/Kaiju", "/wiki/Cavaliere_dell%27Apocalisse", "/wiki/Animale", "/wiki/Goblinoide", "/wiki/Drago_Esterno", "/wiki/Dimensione_del_Tempo", "/wiki/Razze/Munavri"];

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
    
    let (last, _) = get_until(next, "");
    if missed_i == 99 { result_arr.push(last.trim()); }
    else
    { 
        result_arr.insert(missed_i, last.trim());
        let unfilled_values = until.len() - missed_i;
        for i in 0..unfilled_values { result_arr.push(GLOBAL_NULL); } //TODO: Check?
    }
    
    return result_arr;
}

fn add_entry(buf: &mut ByteBuffer, entry_data_str: &str) -> u16
{
    let entry_data = entry_data_str.as_bytes();
    let entry_len  = entry_data_str.len();
    
    let mut cursor: usize = 0;
    
    //NOTE: Index 0 means an empty entry.
    if entry_len == 0 { return cursor as u16; }
    
    //Bullshit bullshit bullshit
    let mut converted_data: Vec<u8> = Vec::with_capacity(entry_len*4);
    for i in 0..entry_len 
    { 
        converted_data.push(entry_data[i]);
        converted_data.push(0);
        converted_data.push(0);
        converted_data.push(0);
    }
    
    cursor = buf.get_wpos();
    
    //NOTE we convert the string to u32 so we need the size to be 4 times larger
    let entry_bytes = entry_len*4;
    buf.write_u8(entry_bytes as u8);
    buf.write_bytes(converted_data.as_slice());
    
    let write_cursor = buf.get_wpos();
    buf.set_wpos(0);
    buf.write_u32((buf.len() - 4) as u32);
    buf.set_wpos(write_cursor);
    
    return cursor as u16;
}

fn add_entry_if_missing(buf: &mut ByteBuffer, entry_data_str: &str) -> u16
{
    let entry_data = entry_data_str.as_bytes();
    let entry_len  = entry_data_str.len();
    
    let mut cursor: usize = 0;
    
    //NOTE: Index 0 means an empty entry.
    if entry_len == 0 { return cursor as u16; }
    
    //NOTE: The first 4 bytes of the buffer represent the total length in bytes
    buf.set_rpos(4);
    while buf.get_rpos() < buf.len()
    {
        cursor         = buf.get_rpos();
        let check_size = buf.read_u8();
        let check_data = buf.read_bytes(check_size as usize);
        
        if entry_data == check_data { return cursor as u16; }
    }
    
    //Bullshit bullshit bullshit
    let mut converted_data: Vec<u8> = Vec::with_capacity(entry_len*4);
    for i in 0..entry_len 
    { 
        converted_data.push(entry_data[i]);
        converted_data.push(0);
        converted_data.push(0);
        converted_data.push(0);
    }
    
    cursor = buf.get_wpos();
    
    //NOTE we convert the string to u32 so we need the size to be 4 times larger
    let entry_bytes = entry_len*4;
    buf.write_u8(entry_bytes as u8);
    buf.write_bytes(converted_data.as_slice());
    
    let write_cursor = buf.get_wpos();
    buf.set_wpos(0);
    buf.write_u32((buf.len() - 4) as u32);
    buf.set_wpos(write_cursor);
    
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    
    use std::time::Instant;
    let now = Instant::now();
    
    let client = reqwest::blocking::Client::new();
    
    let body = client.get("https://golarion.altervista.org/wiki/Database_Mostri").send()?.text()?;
    
    let offset  = body.find("wiki_table_filter").unwrap();
    let slice_1 = body.get((offset - 11)..).unwrap();
    
    let offset_end = slice_1.find("</table>").unwrap();
    let slice_of_page = slice_1.get(..(offset_end+8));
    
    //println!("{}", slice_of_page.unwrap());
    
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
    
    /*
        for p in array_of_paths
        {
            let full_link = base_url.to_string().clone() + &p.to_string();
            println!("{}", full_link);
        }
    */
    
    //let test_index = [1255, 1946, 2170, array_of_paths.len()-1];
    
    //sizeof+char_arr
    let mut number_buffer         = ByteBuffer::from_bytes(&[0u8;64]);
    let mut name_buffer           = ByteBuffer::from_bytes(&[0u8;64]);
    let mut gs_buffer             = ByteBuffer::from_bytes(&[0u8;64]);
    let mut pe_buffer             = ByteBuffer::from_bytes(&[0u8;64]);
    let mut alignment_buffer      = ByteBuffer::from_bytes(&[0u8;64]);
    let mut types_buffer          = ByteBuffer::from_bytes(&[0u8;64]);
    let mut subtypes_buffer       = ByteBuffer::from_bytes(&[0u8;64]);
    let mut sizes_buffer          = ByteBuffer::from_bytes(&[0u8;64]);
    let mut senses_buffer         = ByteBuffer::from_bytes(&[0u8;64]);
    let mut auras_buffer          = ByteBuffer::from_bytes(&[0u8;64]);
    let mut immunities_buffer     = ByteBuffer::from_bytes(&[0u8;64]);
    let mut resistances_buffer    = ByteBuffer::from_bytes(&[0u8;64]);
    let mut weaknesses_buffer     = ByteBuffer::from_bytes(&[0u8;64]);
    let mut special_attack_buffer = ByteBuffer::from_bytes(&[0u8;64]);
    let mut spells_buffer         = ByteBuffer::from_bytes(&[0u8;64]);
    let mut talents_buffer        = ByteBuffer::from_bytes(&[0u8;64]);
    let mut terrains_buffer       = ByteBuffer::from_bytes(&[0u8;64]);
    let mut climates_buffer       = ByteBuffer::from_bytes(&[0u8;64]);
    let mut sources_buffer        = ByteBuffer::from_bytes(&[0u8;64]);
    
    for file_idx in 21..22
        //for file_idx in 0..array_of_paths.len()
    {
        //println!("IDX: {}", file_idx);
        
        let mut mob_string_buffer = ByteBuffer::from_bytes(&[0u8;4]);
        
        let mob_body_opt = client.get(&array_of_paths[file_idx]).send()?.text()?;
        
        //let mut mob_dirty_file = File::create("mob_dirty.html")?;
        //mob_dirty_file.write_all(mob_body_opt.as_bytes())?;
        
        let offset_begin = mob_body_opt.find("<h1>");
        if offset_begin.is_none() {
            println!("[ERROR] Probably non-mob page: {}", array_of_paths[file_idx]);
            return Ok(());
        }
        
        let begin_mob = mob_body_opt.get(offset_begin.unwrap()..).unwrap();
        
        let offset_end = begin_mob.find("<!--").unwrap();
        let mob_page_tmp = begin_mob.get(..offset_end);
        if mob_page_tmp.is_none() { return Ok(()); }
        
        let mob_page_tmp = clear_tag(mob_page_tmp.unwrap(), "<div class=\"toccolours mw-collapsible-content\">", "</div>");
        let mut mob_page = mob_page_tmp.as_str();
        
        let mut tmp_file = File::create("mob.html")?;
        tmp_file.write_all(mob_page.as_bytes())?;
        
        //NOTE: Let's try extracting entire tag blocks to parse the mob data
        let (mob_header, next) = get_slice_inside_tags(mob_page, "<h1>".to_string(), "</h1>".to_string());
        mob_page = next;
        
        let (race_class_info_pre, next) = get_slice_inside_tags(mob_page, "<div class=\"mw-collapsible mw-collapsed\">".to_string(), "</div>".to_string());
        mob_page = next;
        
        let (origin_info, race_class_info) = get_until(&race_class_info_pre, "<br /><i>");
        
        
        let (misc_info, next) = get_slice_inside_tags(mob_page, "<p>".to_string(), "</p>".to_string());
        mob_page = next;
        
        mob_page = skip_to(mob_page, "id=\"Difesa\"");
        let (defense_block, next) = get_slice_inside_tags(mob_page, "<p>".to_string(), "</p>".to_string());
        mob_page = next;
        
        mob_page = skip_to(mob_page, "id=\"Attacco\"");
        let (attack_block, next) = get_slice_inside_tags(mob_page, "<p>".to_string(), "<h2>".to_string());
        mob_page = next;
        
        mob_page = skip_to(mob_page, "id=\"Statistiche\"");
        let (stats_block, next) = get_slice_inside_tags(mob_page, "<p>".to_string(), "</p>".to_string());
        mob_page = next;
        
        let mut specials_block: &str = "";
        
        mob_page = skip_to(mob_page, "id=\"Capacità_Speciali\"");
        if mob_page == GLOBAL_NULL 
        {
            mob_page = skip_to(next, "id=\"Ecologia\">");
        }
        else
        {
            let (specials_block_tmp, next) = get_slice_inside_tags(mob_page, "<h3>".to_string(), "<h2><span class=\"mw-headline\" id=\"Ecologia\">".to_string());
            specials_block = specials_block_tmp;
            mob_page = next;
        }
        
        let (ecology_block, next) = get_slice_inside_tags(mob_page, "<p>".to_string(), "</p>".to_string());
        mob_page = next;
        
        mob_page = skip_to(mob_page, "id=\"Descrizione\"");
        let (desc_block, next) = get_slice_inside_tags(mob_page, "</h2>".to_string(), "<hr />".to_string());
        mob_page = next;
        
        let (source_block, _) = get_slice_inside_tags(mob_page, "<p>".to_string(), "</p>".to_string());
        
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
        
        let head_check    = ["GS", "PE:"];
        let head_arr      = fill_array_from_available(&head, &head_check);
        
        //TODO: Fix all Licanthropes/Entothropes having 2 pages in the same entry.
        //NOTE: Just helping golarion out. If missing shit, I'm gonna report it to fix it.
        if desc.is_empty() { println!("IDX: {}, Name: {}", file_idx, head_arr[0]); }
        
        let class_check   = ["Allineamento: ", "Categoria: ", "(", ")"];
        let mut class_arr = fill_array_from_available(&class, &class_check);
        
        let mut subtypes_count = 0;
        let mut arch_count     = 0;
        
        //NOTE: Manually fix the category block
        if class_arr[3].is_empty()
        {
            
            let size_idx = class_arr[2].rfind(" ");
            if size_idx.is_none() { println!("Maybe error? size_idx is missing. How can it be missing?"); panic!(); }
            
            let size = class_arr[2].get(size_idx.unwrap()..);
            if size.is_none() { println!("How can I not get size??"); panic!(); }
            
            class_arr.push(size.unwrap().trim()); //NOTE: To put it in index 5
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
            
            subtypes_count = flatten_str_list(&mut class_arr, 3+arch_count, ", ");
        }
        
        for v in &class_arr { println!("{}", v); }
        
        //NOTE We differentiate all senses, and from perception we only keep the value
        let misc_check    = ["Sensi:", "Percezione "];
        let mut misc_arr  = fill_array_from_available(&misc, &misc_check);
        
        //NOTE: Manually fix all misc
        let golarion_grammar_mistake = misc_arr[1].find(";"); //NOTE: Fucking golarion has grammar mistakes.
        if golarion_grammar_mistake.is_some() { misc_arr[1] = misc_arr[1].get(..misc_arr[1].len()-1).unwrap(); }
        
        misc_arr[0] = check_unwrap!(misc_arr[0].get(12..), file_idx, head_arr[0]);
        
        let mut senses_count = 0;
        if !misc_arr[1].is_empty() 
        { 
            senses_count = flatten_str_list(&mut misc_arr, 1, ", ");
        }
        
        let defense_check   = ["PF: ", "Tiri Salvezza: ", "RD: ", "RI: ", "Immunità: ", 
                               "Resistenze: ", "Capacità Difensive: ", "Debolezze: "];
        let mut defense_arr = fill_array_from_available(&defense, &defense_check);
        
        let mut immunities_count = 0;
        if !defense_arr[5].is_empty()
        {
            immunities_count = flatten_str_list(&mut defense_arr, 5, ", ");
        }
        
        let mut resitances_count = 0;
        let resistance_offset = if immunities_count > 0 { immunities_count - 1 } else { 0 };
        if !defense_arr[6+resistance_offset].is_empty()
        {
            resitances_count = flatten_str_list(&mut defense_arr, 6+resistance_offset, ", ");
        }
        
        let mut weakness_count = 0;
        let weak_offset = if weakness_count > 0 { (weakness_count - 1) + resistance_offset } else { resistance_offset };
        if !defense_arr[8+weak_offset].is_empty()
        {
            weakness_count = flatten_str_list(&mut defense_arr, 8+weak_offset, ", ");
        }
        
        
        //NOTE: Manually fix AC
        defense_arr[0] = defense_arr[0].get(4..).unwrap();
        
        //TODO Extract LI and Concentration??? Or just print them randomly?
        let attack_check   = ["Mischia:", "Distanza:", "Spazio:", "Portata:",
                              "Attacchi Speciali", "Magia Psichica", "Capacità Magiche", "Incantesimi" ];
        let mut attack_arr = fill_array_from_available(&attack, &attack_check);
        
        //NOTE: Manually fix Speed
        attack_arr[0] = attack_arr[0].get(11..).unwrap();
        
        let stats_check   = ["Bonus di Attacco Base:", "BMC:", "DMC:", "Talenti:", "Abilità:",
                             "Linguaggi:", "Modificatori Razziali:", "Qualità Speciali:" ];
        let mut stats_arr = fill_array_from_available(&stats, &stats_check);
        
        //NOTE: Manually fix Stats
        stats_arr[0] = stats_arr[0].get(17..).unwrap();
        
        let ecology_check   = ["Organizzazione:", "Tesoro:"];
        let mut ecology_arr = fill_array_from_available(&ecology, &ecology_check);
        
        //NOTE: Manually fix Environment
        //TODO: Check if there's a sneaky space before the enviroment tag
        //ecology_arr[0] = ecology_arr[0].get(10..).unwrap();
        ecology_arr[0] = check_unwrap!(ecology_arr[0].get(9..), file_idx, head_arr[0]);
        
        //NOTE: Manually fix source
        source = check_unwrap!(source.get(7..), file_idx, head_arr[0]).to_string();
        
        /*
        //NOTE Start filling the buffers
        {
            let name_idx = add_entry_if_missing(&mut name_buffer, head_arr[0]);
            let gs_idx   = add_entry_if_missing(&mut gs_buffer, head_arr[1]);
            let pe_idx   = add_entry_if_missing(&mut pe_buffer, head_arr[2]);
            
            let origin_idx     = add_entry(&mut mob_string_buffer, origin);
            let short_desc_idx = add_entry(&mut mob_string_buffer, class_arr[0]);
            let align_idx      = add_entry_if_missing(&mut alignment_buffer, class_arr[1]);
            let type_idx       = add_entry_if_missing(&mut types_buffer, class_arr[2]);
            
            let subtypes_idx: [0u16; 4];
            for s in 0..number_of_subtypes
            { subtypes_idx[s]  = add_entry_if_missing(&mut subtypes_buffer, class_arr[3+s]); }
            let size_idx       = add_entry_if_missing(&mut sizes_buffer, class_arr[4+number_of_subtypes]);
            
            let init_idx       = add_entry_if_missing(&mut number_buffer, misc_arr[0]);
            
            let senses_idx: [0u16; 8];
            for s in 0..number_of_senses
            { senses_idx[s]    = add_entry_if_missing(&mut senses_buffer, misc_arr[1+s]); }
            let perception_idx = add_entry_if_missing(&mut number_buffer, misc_arr[2+number_of_senses]);
            
            let ac_idx            = add_entry_if_missing(&mut mob_string_buffer, defense_arr[0]);
            let pf_idx            = add_entry_if_missing(&mut mob_string_buffer, defense_arr[1]);
            let st_idx            = add_entry_if_missing(&mut mob_string_buffer, defense_arr[2]);
            let rd_idx            = add_entry_if_missing(&mut mob_string_buffer, defense_arr[3]);
            let ri_idx            = add_entry_if_missing(&mut mob_string_buffer, defense_arr[4]);
            
            //TODO List
            let immunities_idx    = add_entry_if_missing(&mut immunities_buffer, defense_arr[5+]);
            let resistances_idx   = add_entry_if_missing(&mut resistances_buffer, defense_arr[6+]);
            
            let defensive_cap_idx = add_entry_if_missing(&mut mob_string_buffer, defense_arr[7+]);
            
            //TODO List
            let weaknesses_idx    = add_entry_if_missing(&mut weaknesses_buffer, defense_arr[8+]);
            
        }
        */
        
        /*
        let mut tmp_file = BufWriter::new(OpenOptions::new().append(true).create(true).open("result.txt").unwrap());
        
        tmp_file.write(file_idx.to_string().as_bytes())?;
        tmp_file.write("\n".as_bytes())?;
        for v in head_arr { tmp_file.write(v.as_bytes())?; }
        tmp_file.write("\n".as_bytes())?;
        tmp_file.flush()?;
        */
        
        if head_arr[0] == "Kitsune"
        {
            println!("IDX: {}", file_idx);
            
            for v in head_arr    { if v.is_empty() { println!("Was empty"); continue; } println!("{}", v); }
            
            if !origin.is_empty() { println!("{}", origin); }
            
            for v in class_arr   { if v.is_empty() { println!("Was empty"); continue; } println!("{}", v); }
            for v in misc_arr    { if v.is_empty() { println!("Was empty"); continue; } println!("{}", v); }
            for v in defense_arr { if v.is_empty() { println!("Was empty"); continue; } println!("{}", v); }
            for v in attack_arr  { if v.is_empty() { println!("Was empty"); continue; } println!("{}", v); }
            for v in stats_arr   { if v.is_empty() { println!("Was empty"); continue; } println!("{}", v); }
            
            if !specials.is_empty() { println!("{}", specials); } //NOTE: Specials is already what I want.
            
            for v in ecology_arr { if v.is_empty() { println!("Was empty"); continue; } println!("{}", v); }
            
            if desc.is_empty() { println!("Was empty"); } else { println!("{}", desc); }    //NOTE: Desc is already what I want.
            
            println!("{}", source);   //NOTE: Source is already what I want.
            
            panic!();
        }
        
    }
    
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
    
    Ok(())
}
