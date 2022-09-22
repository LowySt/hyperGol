use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufWriter;
use std::io::prelude::*;

static BLACKLIST: [&str; 57] = ["/wiki/Azata", "/wiki/Agathion", "/wiki/Div", "/wiki/Drago", "/wiki/Demone", "/wiki/Daemon", "/wiki/Arconte", "/wiki/Formian", "/wiki/Demodand", "/wiki/Golem", "/wiki/Diavolo", "/wiki/Calamit%C3%A0", "/wiki/Angelo", "/wiki/Gremlin", "/wiki/Signore_dei_Demoni", "/wiki/Grande_Antico", "/wiki/Dinosauro", "/wiki/Signore_Empireo", "/wiki/Arcidiavolo", "/wiki/Linnorm", "/wiki/Behemoth", "/wiki/Sahkil", "/wiki/Oni", "/wiki/Signore_dei_Qlippoth", "/wiki/Manasaputra", "/wiki/Eone", "/wiki/Asura", "/wiki/Meccanico", "/wiki/Ombra_Notturna", "/wiki/Colosso", "/wiki/Rakshasa", "/wiki/Inevitabile", "/wiki/Caccia_Selvaggia", "/wiki/Sfinge", "/wiki/Thriae", "/wiki/Qlippoth", "/wiki/Psicopompo", "/wiki/Leshy", "/wiki/Popolo_Oscuro", "/wiki/Kami", "/wiki/Kyton", "/wiki/Protean", "/wiki/Razza_Predatrice", "/wiki/Spirito_della_Casa", "/wiki/Tsukumogami", "/wiki/Wysp", "/wiki/Carnideforme", "/wiki/Pesce", "/wiki/Robot", "/wiki/Alveare", "/wiki/Idra", "/wiki/Kaiju", "/wiki/Cavaliere_dell%27Apocalisse", "/wiki/Animale", "/wiki/Goblinoide", "/wiki/Drago_Esterno", "/wiki/Dimensione_del_Tempo"];

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

/*
fn eat_html_tag(mob_data: &str) -> Option<&str> {
	let tag_begin_index = mob_data.find('<');
	if tag_begin_index.is_none() { return None; }
    
	let tag_end_index   = mob_data.find('>');
	if tag_end_index.is_none() { return None; }
    
	return mob_data.get(tag_end_index.unwrap()+1..);
}

fn eat_n_html_tags(mob_data: &str, num: i32) -> Option<&str> {
	let mut prev = mob_data;
	let mut next: Option<&str> = None;
	for _ in 0..num {
		next = eat_html_tag(prev);
		if next.is_none() { return None; }
		
		prev = next.unwrap();
	}
    
	return next;
}

macro_rules! eat_advance_get {
	($mob_page:expr, $n:expr) => {
		{
			let next = eat_n_html_tags(&$mob_page, $n);
			if next.is_none() { return Ok(()); }
            
			$mob_page = next.unwrap();
			let val = $mob_page.get(..$mob_page.find('<').unwrap()).unwrap();
			val
		}
	};
}

fn clear_tags_until(data_slice: &str, delim: char) -> Option<String> {
    
	let check_delim = data_slice.find(delim);
	if check_delim.is_none() { println!("[ERROR] Could not find delimiter {}", delim); return None; }
    
	let result_slice_tmp = data_slice.get(..check_delim.unwrap()+1);
	if result_slice_tmp.is_none() { println!("[ERROR] Could not get until delimiter {}", delim); return None; }
    
	let mut result_slice = String::from(result_slice_tmp.unwrap());
	loop
	{
		let begin = result_slice.find("<");
		if begin.is_none() { return Some(result_slice); }
        
		let end = result_slice.find(">");
		if end.is_none() { println!("[ERROR] Malformed html?"); return None; }
        
		result_slice.replace_range(begin.unwrap()..end.unwrap()+1, "");
	}
}
*/

fn clear_all_tags(data_slice: &str) -> String {
    
	let mut result = String::from(data_slice);
	loop
	{
		let begin = result.find("<");
		if begin.is_none() { return result; }
        
		let end = result.find(">");
		if end.is_none() { println!("[ERROR] Malformed html?"); return "".to_string(); }
        
		result.replace_range(begin.unwrap()..end.unwrap()+1, "");
	}
}

/*
macro_rules! skip_to {
	($mob_page:expr, $tag:expr) => {
		{
			let len = $tag.len();
			let begin = $mob_page.find($tag);
			if begin.is_none() { println!("[ERROR] Couldn't find tag {}", $tag); return Ok(()); }
			
			let val   = $mob_page.get((begin.unwrap() + len)..);
			if val.is_none() { println!("[ERROR] Couldn't slice mob_page past tag {}", $tag); return Ok(()); }
            
			let val2 = val.unwrap();
			val2
		}
	};
}
*/

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
    
    if until.len() == 0 { end_idx = data_slice.len(); }
    else
    { 
        let end_idx_tmp = data_slice.find(until);
        if end_idx_tmp.is_none() { return (GLOBAL_NULL, GLOBAL_NULL); }
        end_idx = end_idx_tmp.unwrap();
    }
    
    let result = data_slice.get(..end_idx);
    if result.is_none() { return (GLOBAL_NULL, GLOBAL_NULL); }
    
    let next = data_slice.get(end_idx..);
    if next.is_none() { return (GLOBAL_NULL, GLOBAL_NULL); }
    
    return (result.unwrap(), next.unwrap());
}

fn get_until_arr<'a>(data_slice: &'a str, until: &[&str], start_idx: usize) -> (&'a str, &'a str, usize)
{
    let mut found = false;
    let mut end_idx: usize = 0;
    let mut idx: usize = 0;
    
    for (i, v) in until[start_idx..].iter().enumerate()
    {
        let index = data_slice.find(v);
        if index.is_some() { found = true; idx = i+1+start_idx; end_idx = index.unwrap(); break; }
    }
    
    if !found { return (GLOBAL_NULL, data_slice, 99); }
    
    let result = data_slice.get(..end_idx);
    if result.is_none() { return (GLOBAL_NULL, data_slice, 99); }
    
    let next = data_slice.get(end_idx..);
    if next.is_none() { return (GLOBAL_NULL, data_slice, 99); }
    
    return (result.unwrap(), next.unwrap(), idx);
}

fn fill_array_from_available<'a>(data_slice: &'a str, until: &[&str]) -> Vec<&'a str>
{
    let mut result_arr = vec![];
    
    let mut next: &str = data_slice;
    let mut el: &str;
    let mut i: usize = 0;
    loop
    {
        (el, next, i)  = get_until_arr(next, until, i);
        if el.is_empty() { break; }
        result_arr.push(el);
    }
    
    let (last, _) = get_until(next, "");
    result_arr.push(last);
    
    return result_arr;
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
    
    for file_idx in 0..100//array_of_paths.len()
    {
        //println!("{}", array_of_paths[83]);
        
        let mob_body_opt = client.get(&array_of_paths[file_idx]).send()?.text()?;
        
        //let mut mob_dirty_file = File::create("mob_dirty.html")?;
        //mob_dirty_file.write_all(mob_body_opt.as_bytes())?;
        
        let offset_begin = mob_body_opt.find("<h1>");
        if offset_begin.is_none() {
            println!("[ERROR] Probably non-mob page: {}", array_of_paths[file_idx]);
            return Ok(());
        }
        
        let begin_mob     = mob_body_opt.get(offset_begin.unwrap()..).unwrap();
        
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
        
        let (race_class_info, next) = get_slice_inside_tags(mob_page, "<div class=\"mw-collapsible mw-collapsed\">".to_string(), "</div>".to_string());
        mob_page = next;
        
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
        let (desc_block, next) = get_slice_inside_tags(mob_page, "<p>".to_string(), "<hr />".to_string());
        mob_page = next;
        
        let (source_block, _) = get_slice_inside_tags(mob_page, "<p>".to_string(), "</p>".to_string());
        
        // NOTE: Now we parse the sub sections.
        let mut specials = String::new();
        
        let head     = clear_all_tags(mob_header);
        let class    = clear_all_tags(race_class_info);
        let misc     = clear_all_tags(misc_info);
        let defense  = clear_all_tags(defense_block);
        let attack   = clear_all_tags(attack_block);
        let stats    = clear_all_tags(stats_block);
        if !specials_block.is_empty() { specials = clear_all_tags(specials_block); }
        let ecology  = clear_all_tags(ecology_block);
        let desc     = clear_all_tags(desc_block);
        let source   = clear_all_tags(source_block);
        
        
        let head_check     = ["GS", "PE"];
        let head_arr   = fill_array_from_available(&head, &head_check);
        
        let class_check    = ["Allineamento:", "Categoria:"];
        let class_arr  = fill_array_from_available(&class, &class_check);
        
        let misc_check     = ["Sensi:"];
        let misc_arr   = fill_array_from_available(&misc, &misc_check);
        
        let defense_check  = ["PF:", "Tiri Salvezza:", "Immunità:", "Capacità Difensive:"];
        let defense_arr    = fill_array_from_available(&defense, &defense_check);
        
        let attack_check   = ["Mischia:", "Distanza:", "Spazio:", "Portata:", 
                              "Attacchi Speciali", "Magia Psichica", "Capacità Magiche", "Incantesimi" ];
        let attack_arr = fill_array_from_available(&attack, &attack_check);
        
        let stats_check    = ["Bonus di Attacco Base:", "BMC:", "DMC:", "Talenti:", "Abilità:", 
                              "Linguaggi:", "Modificatori Razziali:", "Qualità Speciali:" ];
        let stats_arr  = fill_array_from_available(&stats, &stats_check);
        
        
        let ecology_check  = ["Organizzazione:", "Tesoro:"];
        let ecology_arr    = fill_array_from_available(&ecology, &ecology_check);
        
        
        let mut tmp_file = BufWriter::new(OpenOptions::new().append(true).create(true).open("result.txt").unwrap());
        
        tmp_file.write(file_idx.to_string().as_bytes())?;
        tmp_file.write("\n".as_bytes())?;
        for v in head_arr { tmp_file.write(v.as_bytes())?; }
        tmp_file.write("\n".as_bytes())?;
        tmp_file.flush()?;
        
        /*
        {
            
            for v in head_arr    { println!("{}", v); }
            for v in class_arr   { println!("{}", v); }
            for v in misc_arr    { println!("{}", v); }
            for v in defense_arr { println!("{}", v); }
            for v in attack_arr  { println!("{}", v); }
            for v in stats_arr   { println!("{}", v); }
            
            if !specials.is_empty() { println!("{}", specials); } //NOTE: Specials is already what I want.
            
            for v in ecology_arr { println!("{}", v); }
            
            println!("{}", desc);     //NOTE: Desc is already what I want.
            
            println!("{}", source);   //NOTE: Source is already what I want.
        }
        */
        
    }
    
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
    
    Ok(())
}
