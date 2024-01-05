//NOTE: Bit Pattern of an alignment entry:
//
//      [15..8] [7..4] [3..0]
//      ^         ^        ^
//      |         |        |
//      |         |    1st Align
//    Options  2nd Align
//
// Bit 15 indicates whether or not the `(see below)` tag is present
// Bit 14 indicates whether or not the `like creator` tag is present
// Bit 13 indicates whether or not the `Qualsiasi` tag is present
// Bit 12 indicates that 2 possible alignment are present.
//     [Technically superflous, could just check wether 2nd Align == Alignment_Empty, but I have extra bits]
//
// Other bits are currently unused.

const ALIGNMENT_SEE_BELOW:    u16 = 0x8000;
const ALIGNMENT_LIKE_CREATOR: u16 = 0x4000;
const ALIGNMENT_ANY:          u16 = 0x2000;
const ALIGNMENT_OR:           u16 = 0x1000;
const ALIGNMENT_BIT_SIZE:     usize = 4;

const legal_good: u16 = Alignment_Type::Legale_Buono as u16;
const legal_neutral: u16 = Alignment_Type::Legale_Neutrale as u16;
const legal_evil: u16 = Alignment_Type::Legale_Malvagio as u16;

const neutral_good: u16 = Alignment_Type::Neutrale_Buono as u16;
const neutral_neutral: u16 = Alignment_Type::Neutrale_Neutrale as u16;
const neutral_evil: u16 = Alignment_Type::Neutrale_Malvagio as u16;

const caotic_good: u16 = Alignment_Type::Caotico_Buono as u16;
const caotic_neutral: u16 = Alignment_Type::Caotico_Neutrale as u16;
const caotic_evil: u16 = Alignment_Type::Caotico_Malvagio as u16;

#[derive(Debug)]
pub enum Alignment_Type
{
    Alignment_Empty = 0,
    
    Legale_Buono = 1,
    Legale_Neutrale = 2,
    Legale_Malvagio = 3,
    
    Neutrale_Buono = 4,
    Neutrale_Neutrale = 5,
    Neutrale_Malvagio = 6,
    
    Caotico_Buono = 7,
    Caotico_Neutrale = 8,
    Caotico_Malvagio = 9
}

fn map_simple_alignment(align: &str, dbg_original: &str) -> u16
{
    match align {
        "LB" => { return legal_good; }
        "LN" => { return legal_neutral; }
        "LM" => { return legal_evil; }
        
        "NB" => { return neutral_good; }
        "N"  => { return neutral_neutral; }
        "NM" => { return neutral_evil; }
        
        "CB" => { return caotic_good; }
        "CN" => { return caotic_neutral; }
        "CM" => { return caotic_evil; }
        
        "Legale Buono"    => { return legal_good; }
        "Legale Neutrale" => { return legal_neutral; }
        "Legale Malvagio" => { return legal_evil; }
        
        "Neutrale Buono"    => { return neutral_good; }
        "Neutrale Neutrale" => { return neutral_neutral; }
        "Neutrale Malvagio" => { return neutral_evil; }
        
        "Caotico Buono"    => { return caotic_good; }
        "Caotico Neutrale" => { return caotic_neutral; }
        "Caotico Malvagio" => { return caotic_evil; }
        
        _ => { todo!("Can't find simple alignment in {align} from: {dbg_original}"); }
    };
}

//NOTE: The objective is to map alignment without ever needing to rever to a buffer. Let's see if we can do that.
pub fn map_alignment(field: &str, page_name: &str) -> u16
{
    let mut see_below_bit = 0;
    let mut real_field = field;
    if field.contains(" (vedi sotto)") || 
        field.contains(" (ma vedi sotto)") || 
        field.contains(" (vedi testo)")
    {
        see_below_bit = ALIGNMENT_SEE_BELOW;
        let collected: Vec<&str> = field.split(" (").collect();
        real_field = collected[0];
    }
    
    match real_field
    {
        "LB" => { return see_below_bit | legal_good; }
        "LN" => { return see_below_bit | legal_neutral; }
        "LM" => { return see_below_bit | legal_evil; }
        
        "NB" => { return see_below_bit | neutral_good; }
        "N"  => { return see_below_bit | neutral_neutral; }
        "NM" => { return see_below_bit | neutral_evil; }
        
        "CB" => { return see_below_bit | caotic_good; }
        "CN" => { return see_below_bit | caotic_neutral; }
        "CM" => { return see_below_bit | caotic_evil; }
        
        "Legale Buono"    => { return see_below_bit | legal_good; }
        "Legale Neutrale" => { return see_below_bit | legal_neutral; }
        "Legale Malvagio" => { return see_below_bit | legal_evil; }
        
        "Neutrale Buono"    => { return see_below_bit | neutral_good; }
        "Neutrale Neutrale" => { return see_below_bit | neutral_neutral; }
        "Neutrale Malvagio" => { return see_below_bit | neutral_evil; }
        
        "Caotico Buono"    => { return see_below_bit | caotic_good; }
        "Caotico Neutrale" => { return see_below_bit | caotic_neutral; }
        "Caotico Malvagio" => { return see_below_bit | caotic_evil; }
        
        "Lo stesso del suo creatore" => { return see_below_bit | ALIGNMENT_LIKE_CREATOR; }
        "Qualsiasi" => { return see_below_bit | ALIGNMENT_ANY; }
        
        _ => {
            if real_field.contains(" o ")
            {
                let both_align: Vec<&str> = real_field.split(" o ").collect();
                let align_1 = map_simple_alignment(both_align[0], field);
                let align_2 = map_simple_alignment(both_align[1], field);
                
                let final_map: u16 = see_below_bit | ALIGNMENT_OR | (align_1 << ALIGNMENT_BIT_SIZE) | align_2;
                return final_map;
                
                println!("HERE!!!!{page_name}");
            }
            
            if real_field.contains("/")
            {
                let both_align: Vec<&str> = real_field.split("/").collect();
                let align_1 = map_simple_alignment(both_align[0], field);
                let align_2 = map_simple_alignment(both_align[1], field);
                
                let final_map: u16 = see_below_bit | ALIGNMENT_OR | (align_1 << ALIGNMENT_BIT_SIZE) | align_2;
                return final_map;
            }
            
            todo!("{page_name}: {field}");
        }
    }
}

