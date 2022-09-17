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

fn main() -> Result<(), Box<dyn std::error::Error>> {

	let body = reqwest::blocking::get("https://golarion.altervista.org/wiki/Database_Mostri")?.text()?;

	//let offset = body.find("Larva di Calpina").unwrap();
	//let offset_end = body.find("NewPP limit report").unwrap();

	//println!("{:?}", body);

	let offset  = body.find("wiki_table_filter").unwrap();
	let slice_1 = body.get((offset - 11)..).unwrap();
	
	let offset_end = slice_1.find("</table>").unwrap();
	let slice_of_page = slice_1.get(..(offset_end+8));

	//println!("{}", slice_of_page.unwrap());

	let mut array_of_paths = vec![];

	let mut next_slice_index = slice_of_page.unwrap().find("href=");
	let mut next_slice = slice_of_page;
	while next_slice_index.is_some() && next_slice.is_some()
	{
		next_slice = next_slice.unwrap().get((next_slice_index.unwrap() + 1)..);
		if next_slice.is_none() { println!("[ERROR]"); return Ok(()); }

		let page_path   = get_path_from_slice(next_slice.unwrap());
		if page_path.is_some() { array_of_paths.push(page_path.unwrap()); }

		next_slice_index = next_slice.unwrap().find("href=");
	}
	
	let base_url = "https://golarion.altervista.org";

	for p in array_of_paths
	{
		let full_link = base_url.to_string().clone() + &p.to_string();
		println!("{}", full_link);
	}

	Ok(())
}
