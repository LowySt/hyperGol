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
	let mob_body_opt = reqwest::blocking::get(&array_of_paths[1254])?.text()?;

	let offset_begin  = mob_body_opt.find("<h1>").unwrap();
	let begin_mob     = mob_body_opt.get(offset_begin..).unwrap();
	
	let offset_end = begin_mob.find("<!--").unwrap();
	let mob_page = begin_mob.get(..offset_end);
	if mob_page.is_none() { return Ok(()); }
	
	let mut mob_page = mob_page.unwrap();
	//Dirty Mob Parsing
	{

		let name = eat_advance_get!(mob_page, 2);
		println!("Name: {}", name);
		
		let gs = eat_advance_get!(mob_page, 4);
		println!("GS: {}", gs);
		
		let pe = eat_advance_get!(mob_page, 2);
		println!("PE: {}", pe);

		let short_desc = eat_advance_get!(mob_page, 6);
		println!("{}", short_desc);

		let alignment = eat_advance_get!(mob_page, 4);
		println!("Alignment: {}", alignment);

		let typ = eat_advance_get!(mob_page, 4);
		println!("Type: {}", typ);

		let size = eat_advance_get!(mob_page, 1);
		println!("Size: {}", size);

		mob_page = skip_to!(mob_page, "title=\"CA\">");

		let ca = clear_tags_until(mob_page, ')');
		if ca.is_some() { println!("{}", ca.unwrap()); }

		mob_page = skip_to!(mob_page, "title=\"PF\">");

		let pf = clear_tags_until(mob_page, ')');
		if pf.is_some() { println!("{}", pf.unwrap()); }

		mob_page = skip_to!(mob_page, "title=\"Tiri Salvezza\">");
		
		let saving_throws = eat_advance_get!(mob_page, 2);
		println!("Saving Throws: {}", saving_throws);
	}


	//println!("{}", mob_page.unwrap());

	Ok(())
}
