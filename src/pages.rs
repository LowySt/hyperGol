use crate::parse_util::*;

#[derive(Debug)]
pub struct Mob_Page {
    pub page_addr: String,
    
    pub header:  String,
    pub origin:  String,
    pub class:   String,
    pub misc:    String,
    pub defense: String,
    pub attack:  String,
    pub stats:   String,
    pub special: String,
    pub ecology: String,
    pub desc:    String,
    pub source:  String,
}

//TODO: Merge this with mob page in the future
#[derive(Debug)]
pub struct NPC_Page {
    pub page_addr: String,
    
    pub header:  String,
    pub origin:  String,
    pub class:   String,
    pub misc:    String,
    pub defense: String,
    pub attack:  String,
    pub tactics: String,
    pub stats:   String,
    pub special: String,
    pub desc:    String,
    pub source:  String,
}


pub fn get_mob_page(orig_mob_page: &str, page_addr: String) -> (Mob_Page, &str)
{
    let mut mob_page = orig_mob_page;
    
    let (mob_header, next) = get_slice_inside_tags(mob_page, "<h1>", "</h1>");
    mob_page = next;
    
    let (race_class_info_pre, next) = get_slice_inside_tags(mob_page, "<div class=\"mw-collapsible mw-collapsed\">", "</div>");
    mob_page = next;
    
    let (origin_info, race_class_info) = get_until(&race_class_info_pre, "<br /><i>");
    
    let (misc_info, next) = get_slice_inside_tags(mob_page, "<p>", "</p>");
    mob_page = next;
    
    mob_page = skip_to(mob_page, "id=\"Difesa");
    let (defense_block, next) = get_slice_inside_tags(mob_page, "<p>", "</p>");
    mob_page = next;
    
    mob_page = skip_to(mob_page, "id=\"Attacco");
    let (attack_block, next) = get_slice_inside_tags(mob_page, "<p>", "<h2>");
    mob_page = next;
    
    mob_page = skip_to(mob_page, "id=\"Statistiche");
    let (stats_block, next) = get_slice_inside_tags(mob_page, "<p>", "</p>");
    mob_page = next;
    
    let mut specials_block: &str = "";
    
    mob_page = skip_to(mob_page, "id=\"Capacità_Speciali");
    if mob_page == GLOBAL_NULL 
    {
        mob_page = skip_to(next, "id=\"Ecologia");
    }
    else
    {
        let (specials_block_tmp, next) = get_slice_inside_tags(mob_page, "<h3>", "<h2><span class=\"mw-headline\" id=\"Ecologia");
        specials_block = specials_block_tmp;
        mob_page = next;
    }
    
    let (ecology_block, next) = get_slice_inside_tags(mob_page, "<p>", "</p>");
    mob_page = next;
    
    mob_page = skip_to(mob_page, "id=\"Descrizione");
    let (desc_block, next) = get_slice_inside_tags(mob_page, "</h2>", "<hr />");
    mob_page = next;
    
    let (source_block, next) = get_slice_inside_tags(mob_page, "<p>", "</p>");
    
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
    
    let mut res = Mob_Page { page_addr: page_addr,
        header: head, origin: origin, class: class, misc: misc,
        defense: defense, attack: attack, stats: stats, special: specials, 
        ecology: ecology, desc: desc, source: source };
    
    return (res, next.trim());
}

pub fn get_npc_page(orig_npc_page: &str, page_addr: String) -> NPC_Page
{
    let mut npc_page = orig_npc_page;
    
    let (npc_header, next) = get_slice_inside_tags(npc_page, "<h1>", "</h1>");
    npc_page = next;
    
    let (race_class_info_pre, next) = get_slice_inside_tags(npc_page, "<div class=\"mw-collapsible mw-collapsed\">", "</div>");
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
    
    let (misc_info, next) = get_slice_inside_tags(npc_page, "<p>", "</p>");
    npc_page = next;
    
    npc_page = skip_to(npc_page, "id=\"Difesa");
    let (defense_block, next) = get_slice_inside_tags(npc_page, "<p>", "</p>");
    npc_page = next;
    
    npc_page = skip_to(npc_page, "id=\"Attacco");
    let (attack_block, next) = get_slice_inside_tags(npc_page, "<p>", "<h2>");
    npc_page = next;
    
    let mut tactics_block: &str = "";
    npc_page = skip_to(npc_page, "id=\"Tattiche");
    if npc_page != GLOBAL_NULL
    {
        let (tactics_block_tmp, next) = get_slice_inside_tags(npc_page, "<p>", "<h2>");
        tactics_block = tactics_block_tmp;
        npc_page = next;
    }
    else { npc_page = next; }
    
    npc_page = skip_to(npc_page, "id=\"Statistiche");
    let (stats_block, next) = get_slice_inside_tags(npc_page, "<p>", "</p>");
    npc_page = next;
    
    let mut specials_block: &str = "";
    npc_page = skip_to(npc_page, "id=\"Capacità_Speciali");
    if npc_page != GLOBAL_NULL
    {
        let (specials_block_tmp, next) = get_slice_inside_tags(npc_page, "<h3>", "<h2><span class=\"mw-headline\" id=\"Descrizione");
        specials_block = specials_block_tmp;
        npc_page = next;
    }
    else { npc_page = next; }
    
    let mut desc_block: &str = "";
    npc_page = skip_to(npc_page, "id=\"Descrizione");
    if npc_page != GLOBAL_NULL
    {
        let (desc_block_tmp, next) = get_slice_inside_tags(npc_page, "</h2>", "<hr />");
        desc_block = desc_block_tmp;
        npc_page = next;
    }
    else { npc_page = next; }
    
    npc_page = skip_to(npc_page, "<p>Fonte:");
    //let (source_block, next) = get_slice_inside_tags(npc_page, "<p>", "</p>");
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
    
    let mut res = NPC_Page { page_addr: page_addr, 
        header: head, origin: origin, class: class, misc: misc,
        defense: defense, attack: attack, tactics: tactics, 
        stats: stats, special: specials, desc: desc, source: source };
    
    return res;
}

pub fn get_mob_page_array(mob_body_opt: &str, page_path: &str) -> Vec<Mob_Page>
{
    let offset_begin = mob_body_opt.find("<h1>");
    if offset_begin.is_none() {
        println!("[ERROR] Probably non-mob page: {}", page_path);
        return vec![];
        //panic!();
    }
    
    let begin_mob = mob_body_opt.get(offset_begin.unwrap()..).unwrap();
    
    let offset_end = begin_mob.find("<!--").unwrap();
    let mob_page_tmp = begin_mob.get(..offset_end);
    if mob_page_tmp.is_none() { println!("Could not do shit. Not a mob?"); panic!(); }
    
    let mob_page_tmp = clear_tag(mob_page_tmp.unwrap(), "<div class=\"toccolours mw-collapsible-content\"", "</div>");
    
    let mut mob_page = mob_page_tmp.as_str();
    
    //NOTE: Let's try extracting entire tag blocks to parse the mob data
    let mut num_pages = 1;
    let (mut page_one, maybe_next) = get_mob_page(mob_page, page_path.to_string());
    
    let mut pages = Vec::new();
    pages.push(page_one);
    
    let mut page_two: Mob_Page;
    if maybe_next.len() > 3 {
        let maybe_next_tmp = clear_tag(maybe_next, "<div class=\"toccolours mw-collapsible-content\"", "</div>");
        let mut maybe_next = maybe_next_tmp.as_str();
        
        (page_two, _) = get_mob_page(maybe_next, page_path.to_string());
        pages.push(page_two);
    }
    
    return pages;
}

pub fn get_npc_page_array(mob_body_opt: &str, page_path: &str) -> Vec<NPC_Page>
{
    let mut has_two_pages = false;
    
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
    
    let page_one = get_npc_page(&mob_page, page_path.to_string());
    
    let mut pages = Vec::new();
    pages.push(page_one);
    
    if has_two_pages
    {
        let page_two = get_npc_page(&second_page, page_path.to_string());
        pages.push(page_two);
    }
    
    return pages;
}
