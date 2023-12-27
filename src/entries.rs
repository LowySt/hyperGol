use std::time::Instant;
use bytebuffer::ByteBuffer;

use crate::vec_cache::*;
use crate::parse_util::*;
use crate::pages::*;
use crate::skills::*;
use crate::talents::*;

pub const STAT_IDX_IN_ARR:    usize = 0;
pub const BAB_IDX_IN_ARR:     usize = 1;
pub const BMC_IDX_IN_ARR:     usize = 2;
pub const DMC_IDX_IN_ARR:     usize = 3;
pub const TALENTS_IDX_IN_ARR: usize = 4;
pub const SKILLS_IDX_IN_ARR:  usize = 5;
pub const LANG_IDX_IN_ARR:    usize = 6;
pub const RACE_IDX_IN_ARR:    usize = 7;
pub const SPEC_IDX_IN_ARR:    usize = 8;


//TODO: Remove this bullshit
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
pub struct Mob_Entry
{
    pub origin     : u32,            // 4
    pub short_desc : u32,            // 8
    pub ac         : u32,            // 12
    pub pf         : u32,            // 16
    pub st         : u32,            // 20
    pub rd         : u32,            // 24
    pub ri         : u32,            // 28
    pub def_cap    : u32,            // 32
    pub melee      : u32,            // 36
    pub ranged     : u32,            // 40
    pub spec_atk   : u32,            // 44
    pub psych      : u32,            // 48
    pub magics     : u32,            // 52
    pub spells     : u32,            // 56
    pub skills     : [u32; 24],      // 152
    pub talents    : [u32; 24],      // 248
    pub racial_mods: u32,            // 252
    pub spec_qual  : u32,            // 256
    pub specials   : [u32; 24],      // 352
    pub org        : u32,            // 356
    pub treasure   : u32,            // 360
    pub desc       : u32,            // 364
    pub source     : u32,            // 368
    
    pub name       : u16,            // 370
    pub gs         : u16,            // 372
    pub pe         : u16,            // 374 //TODO: Remove this. It's obtainable from GS
    pub align      : u16,            // 376
    pub typ        : u16,            // 378
    pub sub        : [u16; 8],       // 394
    pub arch       : [u16; 4],       // 402
    pub size       : u16,            // 404
    pub init       : u16,            // 406
    pub senses     : [u16; 8],       // 422
    pub perception : u16,            // 424
    pub aura       : u16,            // 426
    pub immunities : [u16; 16],      // 458
    pub resistances: [u16; 16],      // 490
    pub weaknesses : [u16; 16],      // 522
    pub speed      : u16,            // 524
    pub space      : u16,            // 526
    pub reach      : u16,            // 528
    pub str        : u16,            // 530
    pub dex        : u16,            // 532
    pub con        : u16,            // 534
    pub int        : u16,            // 536
    pub wis        : u16,            // 538
    pub cha        : u16,            // 540
    pub bab        : u16,            // 542
    pub cmb        : u16,            // 544
    pub cmd        : u16,            // 546
    pub lang       : [u16; 24],      // 594
    pub env        : u16,            // 596
}

pub static mut create_mob_entry_prepare_time: u128 = 0u128;
pub static mut create_mob_entry_buffer_time: u128  = 0u128;

pub fn create_mob_entry(cache: &mut VectorCache, bufs: &mut Buffer_Context,
                        talentsModule: &mut ByteBuffer, mut page: Mob_Page, file_idx: usize) -> Mob_Entry
{
    //NOTE:Profiliing -------------
    let cme_now = Instant::now();
    //-----------------------------
    
    //TODO: GS Should just be stored as a number
    //      PE Should be completely removed and obtained directly from GS
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
    
    //TODO: Meladaemon Perception and Aura get fucked?
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
    stats_arr[STAT_IDX_IN_ARR] = stats_arr[STAT_IDX_IN_ARR].get(17..).unwrap();
    
    //NOTE: Talents and Skills are not removing from the array, they are copying.
    //      Thus the spacing between sections is unchanged.
    let mut talents_idx = [0u32; 24];
    let mut talent_count = 0;
    if !stats_arr[TALENTS_IDX_IN_ARR].is_empty()
    {
        talents_idx = prepare_talents_str(&mut stats_arr, cache, bufs, talentsModule, &page.page_addr);
    }
    
    let mut skills_idx  = [0u32; 24];
    let mut skill_count = 0;
    let skill_off = if talent_count > 0 { talent_count - 1 } else { 0 };
    if !stats_arr[SKILLS_IDX_IN_ARR+skill_off].is_empty()
    {
        skills_idx = prepare_skill_str(&mut stats_arr, cache, bufs, &page.page_addr, skill_off);
    }
    
    let mut lang_count = 0;
    let lang_off = if skill_count > 0 { (skill_count - 1) + skill_off } else { skill_off };
    if !stats_arr[LANG_IDX_IN_ARR+lang_off].is_empty()
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
    //let mut talents_idx = [0u16; 24];
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
    
    //NOTE: Talents are missing because have been already added
    //NOTE: Skills are  missing because have been already added
    
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
pub struct NPC_Entry
{
    pub origin     : u32,            // 4
    pub short_desc : u32,            // 8
    pub ac         : u32,            // 12
    pub pf         : u32,            // 16
    pub st         : u32,            // 20
    pub rd         : u32,            // 24
    pub ri         : u32,            // 28
    pub def_cap    : u32,            // 32
    pub melee      : u32,            // 36
    pub ranged     : u32,            // 40
    pub spec_atk   : u32,            // 44
    pub psych      : u32,            // 48
    pub magics     : u32,            // 52
    pub spells     : u32,            // 56
    pub tactics    : [u32; 3],       // 68
    pub skills     : [u32; 24],      // 164
    pub talents    : [u32; 24],      // 260
    pub racial_mods: u32,            // 
    pub spec_qual  : u32,            // 
    pub given_equip: u32,            // 
    pub properties : u32,            // 
    pub boons      : u32,            // 
    pub specials   : [u32; 24],      // 
    pub desc       : u32,            // 
    pub source     : u32,            // 
    
    pub name       : u16,            // 
    pub gs         : u16,            // 
    pub pe         : u16,            // 
    pub align      : u16,            // 
    pub typ        : u16,            // 
    pub sub        : [u16; 8],       // 
    pub arch       : [u16; 4],       // 
    pub size       : u16,            // 
    pub init       : u16,            // 
    pub senses     : [u16; 8],       // 
    pub perception : u16,            // 
    pub aura       : u16,            // 
    pub immunities : [u16; 16],      // 
    pub resistances: [u16; 16],      // 
    pub weaknesses : [u16; 16],      // 
    pub speed      : u16,            // 
    pub space      : u16,            // 
    pub reach      : u16,            // 
    pub str        : u16,            // 
    pub dex        : u16,            // 
    pub con        : u16,            // 
    pub int        : u16,            // 
    pub wis        : u16,            // 
    pub cha        : u16,            // 
    pub bab        : u16,            // 
    pub cmb        : u16,            // 
    pub cmd        : u16,            // 
    pub lang       : [u16; 24],      // 610
}

pub fn create_npc_entry(cache: &mut VectorCache, bufs: &mut Buffer_Context, 
                        talentsModule: &mut ByteBuffer, mut page: NPC_Page, file_idx: usize) -> NPC_Entry
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
    
    let mut talents_idx = [0u32; 24];
    let mut talent_count = 0;
    if !stats_arr[TALENTS_IDX_IN_ARR].is_empty()
    {
        //talent_count = flatten_str_list(&mut stats_arr, 4, ", ");
        talents_idx = prepare_talents_str(&mut stats_arr, cache, bufs, talentsModule, &page.page_addr);
    }
    
    let mut skills_idx  = [0u32; 24];
    let mut skill_count = 0;
    let skill_off = if talent_count > 0 { talent_count - 1 } else { 0 };
    if !stats_arr[SKILLS_IDX_IN_ARR+skill_off].is_empty()
    {
        skills_idx = prepare_skill_str(&mut stats_arr, cache, bufs, &page.page_addr, skill_off);
    }
    
    let mut lang_count = 0;
    let lang_off = if skill_count > 0 { (skill_count - 1) + skill_off } else { skill_off };
    if !stats_arr[LANG_IDX_IN_ARR+lang_off].is_empty()
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
    //let mut talents_idx = [0u16; 24];
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
    
    //NOTE: Talents are missing because have been already added
    //NOTE: Skills are  missing because have been already added
    
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



//TODO: Maybe merge this with Mob_Entry in the future??
#[derive(Debug)]
pub struct Talent_Entry
{
    pub name  : u32,
    pub desc  : u32,
    pub pre   : u32,
    pub gain  : u32,
    pub norm  : u32,
    pub spec  : u32,
    pub source: u32,
}

pub const TALENT_ENTRY_SIZE_IN_BYTES: usize = 28;

pub fn create_talent_entry(cache: &mut VectorCache,
                           bufs: &mut Buffer_Context, mut page: Talent_Page, file_idx: usize) -> Talent_Entry
{
    if page.name == "[ERROR]" {
        return Talent_Entry {
            name: 0u32,
            desc: 0u32,
            pre: 0u32,
            gain: 0u32,
            norm: 0u32,
            spec: 0u32,
            source: 0u32,
        };
    }
    
    let name_idx   = add_entry_if_missing_u32(&mut cache.strings, &mut bufs.string, &page.name);
    let desc_idx   = add_entry_if_missing_u32(&mut cache.strings, &mut bufs.string, &page.desc);
    let pre_idx    = add_entry_if_missing_u32(&mut cache.strings, &mut bufs.string, &page.pre);
    let gain_idx   = add_entry_if_missing_u32(&mut cache.strings, &mut bufs.string, &page.gain);
    let norm_idx   = add_entry_if_missing_u32(&mut cache.strings, &mut bufs.string, &page.norm);
    let spec_idx   = add_entry_if_missing_u32(&mut cache.strings, &mut bufs.string, &page.spec);
    let source_idx = add_entry_if_missing_u32(&mut cache.strings, &mut bufs.string, &page.source);
    
    let mut entry = Talent_Entry {
        name: name_idx,
        desc: desc_idx,
        pre: pre_idx,
        gain: gain_idx,
        norm: norm_idx,
        spec: spec_idx,
        source: source_idx,
    };
    
    return entry;
}