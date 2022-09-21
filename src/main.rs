use std::fs::File;
use std::io::prelude::*;

static BLACKLIST: [&str; 53] = ["/wiki/Azata", "/wiki/Agathion", "/wiki/Div", "/wiki/Drago", "/wiki/Demone", "/wiki/Daemon", "/wiki/Arconte", "/wiki/Formian", "/wiki/Demodand", "/wiki/Golem", "/wiki/Diavolo", "/wiki/Calamit%C3%A0", "/wiki/Angelo", "/wiki/Gremlin", "/wiki/Signore_dei_Demoni", "/wiki/Grande_Antico", "/wiki/Dinosauro", "/wiki/Signore_Empireo", "/wiki/Arcidiavolo", "/wiki/Linnorm", "/wiki/Behemoth", "/wiki/Sahkil", "/wiki/Oni", "/wiki/Signore_dei_Qlippoth", "/wiki/Manasaputra", "/wiki/Eone", "/wiki/Asura", "/wiki/Meccanico", "/wiki/Ombra_Notturna", "/wiki/Colosso", "/wiki/Rakshasa", "/wiki/Inevitabile", "/wiki/Caccia_Selvaggia", "/wiki/Sfinge", "/wiki/Thriae", "/wiki/Qlippoth", "/wiki/Psicopompo", "/wiki/Leshy", "/wiki/Popolo_Oscuro", "/wiki/Kami", "/wiki/Kyton", "/wiki/Protean", "/wiki/Razza_Predatrice", "/wiki/Spirito_della_Casa", "/wiki/Tsukumogami", "/wiki/Wysp", "/wiki/Carnideforme", "/wiki/Pesce", "/wiki/Robot", "/wiki/Alveare", "/wiki/Idra", "/wiki/Kaiju", "/wiki/Cavaliere_dell%27Apocalisse"];

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

static GLOBAL_NULL: &str = "";

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    
	let body = reqwest::blocking::get("https://golarion.altervista.org/wiki/Database_Mostri")?.text()?;
    
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
    
    let test_index = [1255, 1946, 2170, array_of_paths.len()-1];
    let mob_body_opt = reqwest::blocking::get(&array_of_paths[test_index[0]])?.text()?;
    
	let offset_begin  = mob_body_opt.find("<h1>").unwrap();
	let begin_mob     = mob_body_opt.get(offset_begin..).unwrap();
	
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
    
    mob_page = skip_to!(mob_page, "id=\"Difesa\"");
    let (defense_block, next) = get_slice_inside_tags(mob_page, "<p>".to_string(), "</p>".to_string());
    mob_page = next;
    
    mob_page = skip_to!(mob_page, "id=\"Attacco\"");
    let (attack_block, next) = get_slice_inside_tags(mob_page, "<p>".to_string(), "<h2>".to_string());
    mob_page = next;
    
    mob_page = skip_to!(mob_page, "id=\"Statistiche\"");
    let (stats_block, next) = get_slice_inside_tags(mob_page, "<p>".to_string(), "</p>".to_string());
    mob_page = next;
    
    mob_page = skip_to!(mob_page, "id=\"Capacità_Speciali\"");
    let (specials_block, next) = get_slice_inside_tags(mob_page, "<h3>".to_string(), "<h2><span class=\"mw-headline\" id=\"Ecologia\">".to_string());
    mob_page = next;
    
    let (ecology_block, next) = get_slice_inside_tags(mob_page, "<p>".to_string(), "</p>".to_string());
    mob_page = next;
    
    mob_page = skip_to!(mob_page, "id=\"Descrizione\"");
    let (desc_block, next) = get_slice_inside_tags(mob_page, "<p>".to_string(), "<hr />".to_string());
    mob_page = next;
    
    let (source_block, _) = get_slice_inside_tags(mob_page, "<p>".to_string(), "</p>".to_string());
    
    /*
    {
        println!("\n{}", mob_header);
        println!("\n{}", race_class_info);
        println!("\n{}", misc_info);
        println!("\n{}", defense);
        println!("\n{}", attack);
        println!("\n{}", stats);
        println!("\n{}", specials);
        println!("\n{}", ecology);
        println!("\n{}", desc);
        println!("\n{}", source);
    }
    */
    
    // NOTE: Now we parse the sub sections.
    let head     = clear_all_tags(mob_header);
    let class    = clear_all_tags(race_class_info);
    let misc     = clear_all_tags(misc_info);
    let defense  = clear_all_tags(defense_block);
    let attack   = clear_all_tags(attack_block);
    let stats    = clear_all_tags(stats_block);
    let specials = clear_all_tags(specials_block);
    let ecology  = clear_all_tags(ecology_block);
    let desc     = clear_all_tags(desc_block);
    let source   = clear_all_tags(source_block);
    
    
    {
        //println!("\nHead: {}", head);
        //println!("\nClass: {}", class);
        //println!("\nMisc: {}", misc);
        //println!("\nDefense: {}", defense);
        //println!("\nAttack: {}", attack);
        //println!("\nStats: {}", stats);
        println!("\nSpecials: {}", specials);
        println!("\nEcology: {}", ecology);
        println!("\nDesc: {}", desc);
        println!("\nSource: {}", source);
    }
    
    let (name, next) = get_until(&head, "GS");
    let (gs, next)   = get_until(next, "PE");
    let (pe, _next)  = get_until(next, "");
    
    let (desc, next)   = get_until(&class, "Allineamento");
    let (align, next)  = get_until(next, "Categoria");
    let (categ, _next) = get_until(next, "");
    
    let (init, next)    = get_until(&misc, "Sensi");
    let (senses, _next) = get_until(next, "");
    
    let (ac, next)          = get_until(&defense, "PF");
    let (pf, next)          = get_until(next, "Tiri Salvezza");
    let (st, next)          = get_until(next, "Immunità");
    let (immunities, next)  = get_until(next, "Capacità Difensive");
    let (def_skills, _next) = get_until(next, "");
    
    
    let mut attack_arr = vec![];
    let element_check = ["Mischia", "Distanza", "Spazio", "Portata", 
                         "Attacchi Speciali", "Magia Psichica", "Capacità Magiche", "Incantesimi" ];
    
    let mut next: &str = &attack;
    let mut el: &str;
    let mut i: usize = 0;
    loop
    {
        (el, next, i)  = get_until_arr(next, &element_check, i);
        if el.is_empty() { break; }
        attack_arr.push(el);
    }
    
    let (last, _) = get_until(next, "");
    attack_arr.push(last);
    
    
    let (scores, next)     = get_until(&stats, "Bonus di Attacco Base");
    let (bab, next)        = get_until(next, "BMC");
    let (cmb, next)        = get_until(next, "DMC");
    let (cmd, next)        = get_until(next, "Talenti");
    let (talents, next)    = get_until(next, "Abilità");
    let (skills, next)     = get_until(next, "Linguaggi");
    let (lang, next)       = get_until(next, "Modificatori Razziali");
    let (race_mod, next)   = get_until(next, "Qualità Speciali");
    let (spec_qual, _next) = get_until(next, "");
    
    
    let mut specials_arr = vec![];
    let mut next: &str = &specials;
    /* Infinite loops. Don't know why?
    loop
    {
        let (v, next) = get_until(next, "\n\n");
        if v.is_empty() { break; }
        specials_arr.push(v);
    }
    */
    {
        println!("{}", name);
        println!("{}", gs);
        println!("{}", pe);
        
        println!("{}", desc);
        println!("{}", align);
        println!("{}", categ);
        
        println!("{}", init);
        println!("{}", senses);
        
        println!("{}", ac);
        println!("{}", pf);
        println!("{}", st);
        println!("{}", immunities);
        println!("{}", def_skills);
        
        for v in attack_arr { println!("{}", v); }
        
        println!("{}", scores);
        println!("{}", bab);
        println!("{}", cmb);
        println!("{}", cmd);
        println!("{}", talents);
        println!("{}", skills);
        println!("{}", lang);
        println!("{}", race_mod);
        println!("{}", spec_qual);
        
        for v in specials_arr { println!("{}", v); }
        
    }
    
    Ok(())
}
