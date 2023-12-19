#![allow(warnings, unused)]

use isahc::prelude::*;

use futures::try_join;
use futures::join;
use futures::executor::block_on;
use std::time::Instant;
use std::thread;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufWriter;
use std::io::prelude::*;
use std::env;
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

pub mod entries;
use crate::entries::*;

pub mod talents;
use crate::talents::*;

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

fn getArrayOfPaths(database_path: &str) -> Vec<String>
{
    let mut body = isahc::get(database_path).unwrap().text().unwrap();
    
    //NOTE: Look for either a wiki_table_filter or a wiki_table_zebra.
    let table_offset = 
        if let Some(found) = body.find("wiki_table_filter") { found } 
    else { body.find("wiki_table_zebra").unwrap() };
    
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
    
    let mut vec_cache = VectorCache::new(4096);
    
    let args: Vec<String> = env::args().collect();
    if args.len() == 1
    {
        let mut total_mob_get_pages_time    = 0u128;
        let mut total_npc_get_pages_time    = 0u128;
        let mut total_mob_create_entry_time = 0u128;
        let mut total_npc_create_entry_time = 0u128;
        
        let mut array_of_paths     = getArrayOfPaths("https://golarion.altervista.org/wiki/Bestiario_di_Golarion");
        let mut array_of_npc_paths = getArrayOfPaths("https://golarion.altervista.org/wiki/Database_PNG");
        
        let garpm_now = Instant::now();
        let mut raw_page_vec = get_all_raw_pages(&array_of_paths);
        total_mob_get_pages_time += garpm_now.elapsed().as_millis();
        
        let mut mob_entries_vec = Vec::with_capacity(array_of_paths.len() + 200);
        let mut npc_entries_vec = Vec::with_capacity(array_of_npc_paths.len() + 200);
        
        //let mut strings_cache = Vec::<CachedIndex>::with_capacity(4096);
        
        let talentModuleFile  = fs::read("TalentsModule").expect("TalentsModule not found or couldn't read!");
        let mut talentsModule = ByteBuffer::from_bytes(&talentModuleFile);
        
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
                let entry = create_mob_entry(&mut vec_cache, &mut buf_context, &mut talentsModule, page, file_idx);
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
            //println!("{file_idx} {:#?}", array_of_npc_paths[file_idx]);
            
            let mut pages = get_npc_page_array(&raw_page_vec[file_idx], &array_of_npc_paths[file_idx]);
            
            for mut page in pages
            {
                let nce_now = Instant::now();
                let entry = create_npc_entry(&mut vec_cache, &mut buf_context, &mut talentsModule, page, file_idx);
                npc_entries_vec.push(entry);
                total_npc_create_entry_time += nce_now.elapsed().as_millis();
            }
        }
        println!("End NPCs");
        
        println!("Mob Count: {}", mob_entries_vec.len());
        println!("NPC Count: {}", npc_entries_vec.len());
        
        let total_size: usize = total_buffers_size(&buf_context);;
        
        println!("Compendium Buffers Size:   {}", total_size);
        println!("TalentsModule Buffer Size: {}", talentsModule.len());
        println!("Total Size of Buffers:     {}", total_size + talentsModule.len());
        
        dump_buffers(&buf_context);
        
        let mut result_file = File::create("Compendium")?;
        
        //NOTE: Write the talents module to the file (buffer + entries)
        result_file.write_all(&talentsModule.to_bytes());
        
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
        
        let byte_padding = [0u8, 0u8];
        
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
            
            //NOTETODO: We are adding 2 bytes of Padding because the C++ structure expects the data to be
            //          SOME byte aligned (maybe 4, maybe 8?), and the next closest byte alignment is 600 bytes
            //         (4 more than the current structure)
            result_file.write_all(&byte_padding);
            result_file.write_all(&byte_padding);
        }
        
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
            //          4 byte aligned, and the next closest byte alignment is 612 bytes
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
    else if args[1] == "talents"
    {
        //TODO: https://golarion.altervista.org/wiki/Talenti_dei_Mostri <----
        //todo!()
        
        let mut paths_1        = getArrayOfPaths("https://golarion.altervista.org/wiki/Talenti_di_Golarion");
        let mut raw_page_vec_1 = get_all_raw_pages(&paths_1);
        
        let mut paths_2        = getArrayOfPaths("https://golarion.altervista.org/wiki/Talenti_dei_Mostri");
        let mut raw_page_vec_2 = get_all_raw_pages(&paths_2);
        
        let mut talent_entries_vec = Vec::with_capacity(paths_1.len() + paths_2.len() + 100);
        
        println!("Begin Talents");
        
        for idx in 0..paths_1.len()
            //for idx in 9..10
        {
            //println!("{idx}: {:#?}", paths_1[idx]);
            let talent_page  = get_talent_page(&raw_page_vec_1[idx], &paths_1[idx]);
            //println!("{:#?}\n", talent_page);
            
            let talent_entry =  create_talent_entry(&mut vec_cache, &mut buf_context, talent_page, idx);
            talent_entries_vec.push(talent_entry);
        }
        
        for idx in 0..paths_2.len()
            //for idx in 9..10
        {
            //println!("{idx}: {:#?}", paths_1[idx]);
            let talent_page  = get_talent_page(&raw_page_vec_2[idx], &paths_2[idx]);
            //println!("{:#?}\n", talent_page);
            
            let talent_entry =  create_talent_entry(&mut vec_cache, &mut buf_context, talent_page, idx);
            talent_entries_vec.push(talent_entry);
        }
        
        println!("End Talents");
        
        
        let total_size: usize = total_buffers_size(&buf_context);;
        println!("Total Size of Buffers:   {}", total_size);
        dump_buffers(&buf_context);
        
        
        let mut result_file = File::create("TalentsModule")?;
        
        //NOTE: Write String Buffer
        result_file.write_all(&buf_context.string.to_bytes());
        
        let size_before_entries = result_file.stream_position().unwrap();
        
        //NOTE: Write Talent Entries
        let talent_vec_len = talent_entries_vec.len() as u32;
        result_file.write_all([talent_vec_len].as_byte_slice());
        
        //TODO: Is there padding?
        for mut entry in talent_entries_vec
        {
            result_file.write_all([entry.name].as_byte_slice());
            result_file.write_all([entry.desc].as_byte_slice());
            result_file.write_all([entry.pre].as_byte_slice());
            result_file.write_all([entry.gain].as_byte_slice());
            result_file.write_all([entry.norm].as_byte_slice());
            result_file.write_all([entry.spec].as_byte_slice());
            result_file.write_all([entry.source].as_byte_slice());
        }
        
        let size_after_entries = result_file.stream_position().unwrap();
        println!("Total Bytes for Entries: {}", size_after_entries - size_before_entries);
        println!("Elapsed:                 {} ms", now.elapsed().as_millis());
        
        Ok(())
    }
    else
    {
        println!("Invalid Argument Passed");
        
        Ok(())
    }
}
