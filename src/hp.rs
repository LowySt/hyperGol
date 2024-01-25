use crate::parse_util::*;

//NOTE: Here's the bit mapping of the u64 HP
//
// [59..54]  [53..47]  [46..36]  [35..27]  [26..18]  [17..9]  [8..0]
//    ^         ^         ^         ^         ^         ^       ^
//    |         |         |         |         |         |     HD (6bit num, 3bit face)
//    |         |         |         |         |         |
//    |         |         |         |         |       HD (6bit num, 3bit face)
//    |         |         |         |         |
//    |         |         |         |       HD (6bit num, 3bit face)  
//    |         |         |         |
//    |         |         |       HD (6bit num, 3bit face)  
//    |         |         |         
//    |         |       Flat value added to life
//    |         |
//    |       Special Kind
//    |
//  Special Kind Value
//
// Bit 63 is negative flat value
// Bit 62 says the creature has a barbarian `Ira`
//
// The 2 bits [61] [60] are currently unused

const HP_MAX_DICE_COUNT: usize = 4;
const HP_DIE_BITLEN: usize = 9;
const HP_DIE_FACE_BITLEN: usize = 3;
const HP_DIE_COUNT_BITLEN: usize = 6;

const HP_OPTION_VAL_MAX: u64 = 64;
const HP_FLAT_OFFSET: usize = 36;
const HP_OPTION_TYPE_OFFSET: usize = 47;
const HP_OPTION_VAL_OFFSET: usize = 54;
const HP_FLAT_NEGATIVE_BIT: u64 = 0x8000000000000000;
const HP_IRA_BIT:           u64 = 0x4000000000000000;

const HP_INVALID: u64 = 0xFFFFFFFFFFFFFFFF;

enum HP_Die_Face
{
    HP_Die_Invalid = 0,
    
    HP_Die_D4 = 1,
    HP_Die_D6 = 2,
    HP_Die_D8 = 3,
    HP_Die_D10 = 4,
    HP_Die_D12 = 5,
}

#[derive(Debug)]
pub enum HP_Options_Type
{
    HP_Invalid = 0,
    
    HP_Guarigione_Rapida,
    HP_Guarigione_Rapida_Ghiaccio_Neve,
    HP_Guarigione_Rapida_Fuoco,
    HP_Guarigione_Rapida_Magma_Lava,
    HP_Guarigione_Rapida_Aridi,
    HP_Guarigione_Rapida_Acqua_Bollente_Vapore,
    HP_Guarigione_Rapida_Sotto_Acqua,
    HP_Guarigione_Rapida_Acqua_Salata,
    HP_Guarigione_Rapida_Tempesta_Vento,
    HP_Guarigione_Rapida_Umidi_Fangosi,
    HP_Guarigione_Rapida_Polverosi,
    HP_Guarigione_Rapida_Sotto_Terra,
    HP_Guarigione_Rapida_Ombre,
    
    HP_Guarigione_Rapida_Planare,
    
    HP_Guarigione_Sangue,
    
    HP_Rigenerazione,
    HP_Rigenerazione_Fuoco,
    HP_Rigenerazione_Elettricita,
    HP_Rigenerazione_Acido,
    HP_Rigenerazione_Freddo,
    HP_Rigenerazione_Terra,
    HP_Rigenerazione_Aria,
    HP_Rigenerazione_Acido_Mitico,
    HP_Rigenerazione_Acido_Sonoro,
    HP_Rigenerazione_Acido_Fuoco,
    HP_Rigenerazione_Acido_Fuoco_In_Acqua,
    HP_Rigenerazione_Acido_Freddo,
    HP_Rigenerazione_Acido_Freddo_Fuoco,
    HP_Rigenerazione_Contundente_Fuoco,
    HP_Rigenerazione_Terreno_Naturale,
    HP_Rigenerazione_Caos,
    HP_Rigenerazione_Caos_Magia,
    HP_Rigenerazione_Fuoco_Bene,
    HP_Rigenerazione_Bene,
    HP_Rigenerazione_Bene_Argento,
    HP_Rigenerazione_Bene_Ruggine,
    HP_Rigenerazione_Male,
    HP_Rigenerazione_Legale,
    HP_Rigenerazione_Mitico,
    HP_Rigenerazione_Deifico_Mitico,
    HP_Rigenerazione_Epico,
    HP_Rigenerazione_Bene_Epico_Mitico,
    HP_Rigenerazione_Bene_Deifico_Mitico,
    HP_Rigenerazione_Energia_Positiva,
    HP_Rigenerazione_Energia_Positiva_Ispirazione,
    HP_Rigenerazione_Energia_Negativa,
    HP_Rigenerazione_Freddo_Male,
    HP_Rigenerazione_Ferro_Freddo,
    HP_Rigenerazione_Adamantio,
    HP_Rigenerazione_No_Armi,
    HP_Rigenerazione_Armi_Maledette,
    HP_Rigenerazione_Armi_Epiche,
    HP_Rigenerazione_Pietrificazione,
    HP_Rigenerazione_Statuetta,
    HP_Rigenerazione_Artefatti_Malvagi,
    HP_Rigenerazione_Speciale,
    
    HP_Regen_Pazuzu,
    
    HP_Rigenerazione_Cera_Fuoco,
    
    HP_Rigenerazione_Pluviale_Acido_Fuoco,
    
    HP_Rigenerazione_Profana_Bene_Elettricita,
    
    HP_Ripararsi_Con_Rifiuti,
    
    HP_Terribile_Ringiovanimento,
    
    HP_Campo_Forza,
    
    HP_Vita_Falsata,
    
    HP_Raccolto_Dolore,
    
    HP_Divorare_Anima,
    
    HP_Estorsione,
    
    HP_Gioia_Sostenente,
}

enum HpExpr
{
    Hp_Error,
    
    Hp_Die(u64),
    Hp_Flat(u64),
    Hp_Flat_False(u64, u64),
}


fn map_hp_expr(field: &str, page_name: &str) -> HpExpr
{
    if let Some(idx) = field.find('d') {
        let dice_count = field[..idx].parse::<u16>()
            .unwrap_or_else(|_| panic!("Couldn't parse dice count value mapping hp dice in {field} from {page_name}"));
        
        //NOTE: This is valid because 'd' is a 1 byte utf8 codepoint.
        let dice_face = field[idx+1..].parse::<u16>()
            .unwrap_or_else(|_| panic!("Couldn't parse dice face value mapping hp dice in {field} from {page_name}"));
        
        let dice_face_idx: u16 = match dice_face {
            4 => { HP_Die_Face::HP_Die_D4   as u16 }
            6 => { HP_Die_Face::HP_Die_D6   as u16 }
            8 => { HP_Die_Face::HP_Die_D8   as u16 }
            10 => { HP_Die_Face::HP_Die_D10 as u16 }
            12 => { HP_Die_Face::HP_Die_D12 as u16 }
            
            _ => { todo!("Unhandled die face in {field} from {page_name}"); }
        };
        
        let HD: u16 = ((dice_count & 0x3F) << HP_DIE_FACE_BITLEN) | (dice_face_idx & 0x0007);
        return HpExpr::Hp_Die(HD as u64);
    }
    else {
        //NOTE: If there's no d, it should be a flat value...
        
        if let Some(idx) = field.find("Vita Falsata") {
            if let Some(plus_idx) = field.find("pi\u{00F9}") {
                //NOTE found something like `più XX Vita Falsata`
                let piu_len = "pi\u{00F9}".len();
                
                let flat = field[..plus_idx].trim().parse::<u16>()
                    .unwrap_or_else(|_| panic!("Couldn't parse flat from Vita Falsata in {field} from {page_name}"));
                
                assert!(flat < 1024, "10 Bits are not enough for flat");
                assert!(flat < 2048, "11 Bits are not enough for flat");
                assert!(flat < 4096, "12 Bits are not enough for flat");
                
                let opt_val = field[plus_idx+piu_len..idx].trim().parse::<u16>()
                    .unwrap_or_else(|_| panic!("Couldn't parse Vita Falsata val in {field} from {page_name}"));
                
                return HpExpr::Hp_Flat_False(flat as u64, opt_val as u64);
            }
            
            todo!("Unhandled vita falsata parsing");
        }
        
        let flat = field.trim().parse::<u16>()
            .unwrap_or_else(|_| panic!("Couldn't parse possibly flat value in {field} from {page_name}"));
        
        assert!(flat < 1024, "10 Bits are not enough for flat");
        assert!(flat < 2048, "11 Bits are not enough for flat");
        assert!(flat < 4096, "12 Bits are not enough for flat");
        
        return HpExpr::Hp_Flat(flat as u64);
    }
    
    panic!("Parsing of hp die failed in: {field} from {page_name}");
    return HpExpr::Hp_Error;
}

pub fn map_or_intern_hp(field: &str, page_name: &str) -> u64
{
    let mut final_value: u64 = 0;
    
    //NOTE: Skip the total value, which won't be stored, and will be recalculated by PCMan
    let mut real_field: &str = field;
    if let Some(idx) = field.find("(") {
        if let Some(substr) = field.get(idx..) {
            real_field = substr.trim_start_matches("(");
        }
        else { panic!("Couldn't slice substr in hp"); }
    }
    else { panic!("Couldn't find substr in hp"); }
    
    //NOTE: Remove `X DV;` & `X DV:` which is completely useless info.
    if let Some(idx) = real_field.find("DV;") {
        let len = "DV;".len();
        if let Some(substr) = real_field.get(idx+len..) {
            real_field = substr.trim();
        }
        else { panic!("Couldn't slice substr when removing DV; in hp from: {field} in {page_name}"); }
    }
    if let Some(idx) = real_field.find("DV:") {
        let len = "DV:".len();
        if let Some(substr) = real_field.get(idx+len..) {
            real_field = substr.trim();
        }
        else { panic!("Couldn't slice substr when removing DV: in hp from: {field} in {page_name}"); }
    }
    
    //NOTE: Remove the `Ira` specs, and set a bit in the final_value
    if let Some(idx) = real_field.find("\nPF non in ira") {
        if let Some(substr) = real_field.get(..idx) {
            real_field = substr.trim();
            final_value |= HP_IRA_BIT;
        }
        else { panic!("Couldn't slice substr when removing ira in hp from: {field} in {page_name}"); }
    }
    
    let mut is_vita_falsata = false;
    let mut vita_falsata_val: u64 = 0;
    
    //NOTE: Now we start parsing the hit dice and the flat value
    let mut dice_count = 0;
    let mut total_flat: i64 = 0;
    let mut remap_start_idx = false;
    let mut value_start_idx = 0;
    let mut char_iter = real_field.char_indices();
    while let Some((base_idx, char)) = char_iter.next()
    {
        if remap_start_idx == true { value_start_idx = base_idx; remap_start_idx = false; }
        
        match char
        {
            //NOTETODO: I think whatever comes before this is a die roll?
            '+' => {
                remap_start_idx = true;
                
                match map_hp_expr(&real_field[value_start_idx..base_idx].trim(), page_name) {
                    
                    HpExpr::Hp_Die(dice_mapping) => {
                        assert!(dice_count < HP_MAX_DICE_COUNT, "Too many dice for {field} in {page_name}!");
                        final_value |= (dice_mapping << (HP_DIE_BITLEN*dice_count));
                        dice_count += 1;
                    }
                    
                    HpExpr::Hp_Flat(val_mapping) => { total_flat += val_mapping as i64; }
                    
                    HpExpr::Hp_Flat_False(flat, opt_val) => {
                        is_vita_falsata = true;
                        vita_falsata_val = opt_val;
                        total_flat += flat as i64;
                    }
                    
                    _ => { todo!() }
                }
                
            }
            
            '-' | '\u{2013}' => {
                remap_start_idx = true;
                
                match map_hp_expr(&real_field[value_start_idx..base_idx].trim(), page_name) {
                    
                    HpExpr::Hp_Die(dice_mapping) => {
                        assert!(dice_count < HP_MAX_DICE_COUNT, "Too many dice for {field} in {page_name}!");
                        final_value |= (dice_mapping << (HP_DIE_BITLEN*dice_count));
                        dice_count += 1;
                    }
                    
                    HpExpr::Hp_Flat(val_mapping) => { total_flat -= val_mapping as i64; }
                    
                    HpExpr::Hp_Flat_False(flat, opt_val) => {
                        is_vita_falsata = true;
                        vita_falsata_val = opt_val;
                        total_flat -= flat as i64;
                    }
                    
                    _ => { todo!() }
                }
            }
            
            //NOTETODO: I think since we've reached the end, we've got either a flat value or nothing.
            ')' => {
                
                match map_hp_expr(&real_field[value_start_idx..base_idx].trim(), page_name) {
                    HpExpr::Hp_Die(dice_mapping) => {
                        assert!(dice_count < HP_MAX_DICE_COUNT, "Too many dice for {field} in {page_name}!");
                        final_value |= (dice_mapping << (HP_DIE_BITLEN*dice_count));
                        dice_count += 1;
                        break;
                    }
                    
                    //NOTETODO: This should never happen... but what do I know?
                    HpExpr::Hp_Flat(val_mapping) => { total_flat += val_mapping as i64; break; }
                    
                    HpExpr::Hp_Flat_False(flat, opt_val) => {
                        is_vita_falsata = true;
                        vita_falsata_val = opt_val;
                        total_flat += flat as i64;
                        break;
                    }
                    
                    _ => { todo!() }
                }
            }
            
            _ => {}
        }
    }
    
    //NOTE: Set the flat value which we may have accumulated during the parsing.
    if total_flat < 0 { 
        let positive_flat: u64 = (-total_flat) as u64;
        final_value |= HP_FLAT_NEGATIVE_BIT;
        final_value |= (positive_flat << HP_FLAT_OFFSET);
    }
    else { final_value |= ((total_flat as u64) << HP_FLAT_OFFSET); }
    
    if(is_vita_falsata == true) {
        final_value |= ((HP_Options_Type::HP_Vita_Falsata as u64) << HP_OPTION_TYPE_OFFSET);
        final_value |= (vita_falsata_val << HP_OPTION_VAL_OFFSET);
        
        assert!(vita_falsata_val < HP_OPTION_VAL_MAX, "Option Val is too large in {field} for {page_name}");
        
        real_field = char_iter.as_str().trim();
        if real_field.len() < 3 { return final_value; }
        
        panic!("Something other than vita falsata!")
    }
    
    //NOTE: Now we check if there's anything left after the main HP formula, for optional capacities.
    real_field = char_iter.as_str().trim();
    if real_field.len() < 3 { return final_value; }
    
    let trim_check: &[_] = &[';', ',', ' '];
    real_field = real_field.trim_matches(trim_check);
    
    //NOTE: I don't understand why I would care about this specific Kineticist Class ability on HP??
    //      It's something the GM would have to keep track of anyway!?!?
    if real_field.contains("Danni Non Letali da bruciare") {
        return final_value;
    }
    
    if real_field.contains("Guarigione Rapida Planare") {
        real_field = real_field.trim_start_matches("Guarigione Rapida Planare").trim();
        
        //NOTE: Find the end of the value
        let mut value_end_idx = 0;
        let mut val_iter = real_field.char_indices();
        while let Some((val_idx, char)) = val_iter.next() {
            if !char.is_digit(10) { value_end_idx = val_idx; break; }
        }
        
        let num_field = if value_end_idx > 0 { &real_field[..value_end_idx] } else { &real_field };
        
        let option_val: u64 = num_field.parse::<u64>()
            .unwrap_or_else(|_| panic!("Invalid number in Guarigione Rapida HP mapping in {num_field} from {field}"));
        
        assert!(option_val < HP_OPTION_VAL_MAX, "Option Val is too large in {field} for {page_name}");
        
        real_field = real_field[value_end_idx..].trim();
        
        //NOTE: Now find the specific kind of regen.
        let option_type: u64 = match real_field {
            _ => {
                if real_field.len() < 3 { HP_Options_Type::HP_Guarigione_Rapida_Planare as u64 }
                else { todo!("Unhandled Guarigione Rapida Planare Type from {field}"); }
            }
        };
        
        final_value |= (option_type << HP_OPTION_TYPE_OFFSET);
        final_value |= (option_val << HP_OPTION_VAL_OFFSET);
        
        return final_value;
    }
    
    if real_field.contains("Guarigione dal Sangue") {
        let option_type: u64 = HP_Options_Type::HP_Guarigione_Sangue as u64;
        final_value |= (option_type << HP_OPTION_TYPE_OFFSET);
        
        return final_value;
    }
    
    if real_field.contains("Campo di Forza") {
        let option_type: u64 = HP_Options_Type::HP_Campo_Forza as u64;
        final_value |= (option_type << HP_OPTION_TYPE_OFFSET);
        
        return final_value;
    }
    
    if real_field.contains("Terribile Ringiovanimento") {
        real_field = real_field.trim_start_matches("Terribile Ringiovanimento").trim();
        
        //NOTE: Find the end of the value
        let mut value_end_idx = 0;
        let mut val_iter = real_field.char_indices();
        while let Some((val_idx, char)) = val_iter.next() {
            if !char.is_digit(10) { value_end_idx = val_idx; break; }
        }
        
        let num_field = if value_end_idx > 0 { &real_field[..value_end_idx] } else { &real_field };
        
        let option_val: u64 = num_field.parse::<u64>()
            .unwrap_or_else(|_| panic!("Invalid number Terribile Ringiovanimento HP map in {num_field} from {field}"));
        
        assert!(option_val < HP_OPTION_VAL_MAX, "Option Val is too large in {field} for {page_name}");
        
        real_field = real_field[value_end_idx..].trim();
        
        //NOTE: Now find the specific kind of regen.
        let option_type: u64 = match real_field {
            _ => {
                if real_field.len() < 3 { HP_Options_Type::HP_Terribile_Ringiovanimento as u64 }
                else { todo!("Unhandled Terribile Ringiovanimento Planare Type from {field}"); }
            }
        };
        
        final_value |= (option_type << HP_OPTION_TYPE_OFFSET);
        final_value |= (option_val << HP_OPTION_VAL_OFFSET);
        
        return final_value;
    }
    
    if real_field.contains("Guarigione Rapida") {
        real_field = real_field.trim_start_matches("Guarigione Rapida").trim();
        
        //NOTE: Find the end of the value
        let mut value_end_idx = 0;
        let mut val_iter = real_field.char_indices();
        while let Some((val_idx, char)) = val_iter.next() {
            if !char.is_digit(10) { value_end_idx = val_idx; break; }
        }
        
        let num_field = if value_end_idx > 0 { &real_field[..value_end_idx] } else { &real_field };
        
        let option_val: u64 = num_field.parse::<u64>()
            .unwrap_or_else(|_| panic!("Invalid number in Guarigione Rapida HP mapping in {num_field} from {field}"));
        
        assert!(option_val < HP_OPTION_VAL_MAX, "Option Val is too large in {field} for {page_name}");
        
        //NOTE: Stupid exception because `Fauci_della_Madre` is the only creature which presents 2 possible values
        //      For Guarigione Rapida, rather than just the base value, and leave the change up to the DM
        //      When using the ability `Divorare Anima`
        if let Some(paren_idx) = real_field.find("(") { real_field = real_field[paren_idx..].trim(); }
        else                                          { real_field = real_field[value_end_idx..].trim(); }
        
        //NOTE: Now find the specific kind of regen.
        let option_type: u64 = match real_field {
            "(solo se a contatto di ghiaccio e neve)" => 
            { HP_Options_Type::HP_Guarigione_Rapida_Ghiaccio_Neve as u64 }
            "(Funziona solo in zone coperte dal ghiaccio)" => 
            { HP_Options_Type::HP_Guarigione_Rapida_Ghiaccio_Neve as u64 }
            
            "(nel fuoco)" => { HP_Options_Type::HP_Guarigione_Rapida_Fuoco as u64 }
            "(Funziona solo a contatto col fuoco)" => { HP_Options_Type::HP_Guarigione_Rapida_Fuoco as u64 }
            "(Funziona solo a contatto con magma o lava)" => 
            { HP_Options_Type::HP_Guarigione_Rapida_Magma_Lava as u64 }
            "(Funziona solo in ambienti aridi)" => { HP_Options_Type::HP_Guarigione_Rapida_Aridi as u64 }
            "(Funziona solo nell'acqua bollente o nel vapore)" =>
            { HP_Options_Type::HP_Guarigione_Rapida_Acqua_Bollente_Vapore as u64 }
            "(Funziona solo sott'acqua)" =>
            { HP_Options_Type::HP_Guarigione_Rapida_Sotto_Acqua as u64 }
            "(solo in acqua salata)" =>
            { HP_Options_Type::HP_Guarigione_Rapida_Acqua_Salata as u64 }
            "(Funziona solo in zone tempestose e ventose)" =>
            { HP_Options_Type::HP_Guarigione_Rapida_Tempesta_Vento as u64 }
            "(Funziona solo in ambienti umidi e fangosi)" =>
            { HP_Options_Type::HP_Guarigione_Rapida_Umidi_Fangosi as u64 }
            "(Funziona solo in ambienti polverosi)" =>
            { HP_Options_Type::HP_Guarigione_Rapida_Polverosi as u64 }
            "(Funziona solo sottoterra)" =>
            { HP_Options_Type::HP_Guarigione_Rapida_Sotto_Terra as u64 }
            "(solo fra le ombre)" =>
            { HP_Options_Type::HP_Guarigione_Rapida_Ombre as u64 }
            
            "(vedi sotto)" => { HP_Options_Type::HP_Guarigione_Rapida as u64 }
            "(vedi Forma Vivente)" => { HP_Options_Type::HP_Guarigione_Rapida as u64 }
            "(vedi Raccolto di Dolore)" => { HP_Options_Type::HP_Raccolto_Dolore as u64 }
            "(vedi Divorare Anima)" => { HP_Options_Type::HP_Divorare_Anima as u64  }
            "(vedi Estorsione)" => { HP_Options_Type::HP_Estorsione as u64  }
            
            _ => {
                if real_field.len() < 3 { HP_Options_Type::HP_Guarigione_Rapida as u64 }
                else { todo!("Unhandled Guarigione Rapida Type from {field} in {page_name}"); }
            }
        };
        
        final_value |= (option_type << HP_OPTION_TYPE_OFFSET);
        final_value |= (option_val << HP_OPTION_VAL_OFFSET);
        
        return final_value;
    }
    
    if real_field.contains("Rigenerazione della Cera") {
        real_field = real_field.trim_start_matches("Rigenerazione della Cera").trim();
        
        //NOTE: Find the end of the regen value
        let mut value_end_idx = 0;
        let mut val_iter = real_field.char_indices();
        while let Some((val_idx, char)) = val_iter.next() {
            if !char.is_digit(10) { value_end_idx = val_idx; break; }
        }
        
        let num_field = &real_field[..value_end_idx];
        
        let option_val: u64 = num_field.parse::<u64>()
            .unwrap_or_else(|_| panic!("Invalid number in Rigenerazione in HP mapping in {num_field} from {field}"));
        
        assert!(option_val < HP_OPTION_VAL_MAX, "Option Val is too large in {field} for {page_name}");
        
        real_field = real_field[value_end_idx..].trim();
        
        //NOTE: Now find the specific kind of regen.
        let option_type: u64 = match real_field {
            "(fuoco)" => { HP_Options_Type::HP_Rigenerazione_Cera_Fuoco as u64 }
            
            _ => { todo!("Unhandled Rigenerazione Type from {field}"); }
        };
        
        final_value |= (option_type << HP_OPTION_TYPE_OFFSET);
        final_value |= (option_val << HP_OPTION_VAL_OFFSET);
        
        return final_value;
    }
    
    if real_field.contains("Rigenerazione Pluviale") {
        real_field = real_field.trim_start_matches("Rigenerazione Pluviale").trim();
        
        //NOTE: Find the end of the regen value
        let mut value_end_idx = 0;
        let mut val_iter = real_field.char_indices();
        while let Some((val_idx, char)) = val_iter.next() {
            if !char.is_digit(10) { value_end_idx = val_idx; break; }
        }
        
        let num_field = &real_field[..value_end_idx];
        
        let option_val: u64 = num_field.parse::<u64>()
            .unwrap_or_else(|_| panic!("Invalid number in Rigenerazione in HP mapping in {num_field} from {field}"));
        
        assert!(option_val < HP_OPTION_VAL_MAX, "Option Val is too large in {field} for {page_name}");
        
        real_field = real_field[value_end_idx..].trim();
        
        //NOTE: Now find the specific kind of regen.
        let option_type: u64 = match real_field {
            "(acido o fuoco)" => { HP_Options_Type::HP_Rigenerazione_Pluviale_Acido_Fuoco as u64 }
            
            _ => { todo!("Unhandled Rigenerazione Type from {field}"); }
        };
        
        final_value |= (option_type << HP_OPTION_TYPE_OFFSET);
        final_value |= (option_val << HP_OPTION_VAL_OFFSET);
        
        return final_value;
    }
    
    if real_field.contains("Rigenerazione Profana") {
        real_field = real_field.trim_start_matches("Rigenerazione Profana").trim();
        
        //NOTE: Find the end of the regen value
        let mut value_end_idx = 0;
        let mut val_iter = real_field.char_indices();
        while let Some((val_idx, char)) = val_iter.next() {
            if !char.is_digit(10) { value_end_idx = val_idx; break; }
        }
        
        let num_field = &real_field[..value_end_idx];
        
        let option_val: u64 = num_field.parse::<u64>()
            .unwrap_or_else(|_| panic!("Invalid number in Rigenerazione in HP mapping in {num_field} from {field}"));
        
        assert!(option_val < HP_OPTION_VAL_MAX, "Option Val is too large in {field} for {page_name}");
        
        real_field = real_field[value_end_idx..].trim();
        
        //NOTE: Now find the specific kind of regen.
        let option_type: u64 = match real_field {
            "(elettricit\u{00E0} o bene)" => { HP_Options_Type::HP_Rigenerazione_Profana_Bene_Elettricita as u64 }
            
            _ => { todo!("Unhandled Rigenerazione Type from {field}"); }
        };
        
        final_value |= (option_type << HP_OPTION_TYPE_OFFSET);
        final_value |= (option_val << HP_OPTION_VAL_OFFSET);
        
        return final_value;
    }
    
    if real_field.contains("Ripararsi con i Rifiuti") {
        real_field = real_field.trim_start_matches("Ripararsi con i Rifiuti").trim();
        if real_field.len() > 3 { todo!("Unhandled data after Ripararsi con i Rifiuti from {field} in {page_name}"); }
        
        let option_type: u64 = HP_Options_Type::HP_Ripararsi_Con_Rifiuti as u64;
        final_value |= (option_type << HP_OPTION_TYPE_OFFSET);
        
        return final_value;
    }
    
    if real_field.contains("Gioia Sostenente") {
        real_field = real_field.trim_start_matches("Gioia Sostenente").trim();
        if real_field.len() > 3 { todo!("Unhandled data after Gioia Sostenente from {field} in {page_name}"); }
        
        let option_type: u64 = HP_Options_Type::HP_Gioia_Sostenente as u64;
        final_value |= (option_type << HP_OPTION_TYPE_OFFSET);
        
        return final_value;
    }
    
    //NOTE: Pazuzu just fucking doesn't have a regen value. It must be an errata, but nobody at Paizo 
    //      gave a shit apparently.
    //
    //      According to the `Core Rule Book Errata` page 461, Regeneration has effects regardless of
    //      the amount of hitpoints recovered. Which might mean Pazuzu doesn't regen any hitpoints,
    //      it just can't be killed unless inflicted with deific or mithic damage (or brought to CON 0)
    if real_field == "Rigenerazione (deifico o mitico)" {
        let option_type: u64 = HP_Options_Type::HP_Regen_Pazuzu as u64;
        final_value |= (option_type << HP_OPTION_TYPE_OFFSET);
        
        return final_value;
    }
    
    if real_field.contains("Rigenerazione") {
        real_field = real_field.trim_start_matches("Rigenerazione").trim();
        
        //NOTE: Find the end of the regen value
        let mut value_end_idx = 0;
        let mut val_iter = real_field.char_indices();
        while let Some((val_idx, char)) = val_iter.next() {
            if !char.is_digit(10) { value_end_idx = val_idx; break; }
        }
        
        let num_field = if value_end_idx > 0 { &real_field[..value_end_idx] } else { &real_field };
        
        let option_val: u64 = num_field.parse::<u64>()
            .unwrap_or_else(|_| panic!("Invalid number Rigenerazione in HP map {num_field} from {field} {page_name}"));
        
        assert!(option_val < HP_OPTION_VAL_MAX, "Option Val is too large in {field} for {page_name}");
        
        real_field = real_field[value_end_idx..].trim();
        
        //NOTE: Now find the specific kind of regen.
        let option_type: u64 = match real_field {
            "(fuoco)" => { HP_Options_Type::HP_Rigenerazione_Fuoco as u64 }
            "(fuoco, vedi Vigore Primordiale)" => { HP_Options_Type::HP_Rigenerazione_Fuoco as u64 }
            "(elettricit\u{00E0})" => { HP_Options_Type::HP_Rigenerazione_Elettricita as u64 }
            "(elettricit\u{00E0}, vedi sotto)" => { HP_Options_Type::HP_Rigenerazione_Elettricita as u64 }
            "(su terreno naturale)" => { HP_Options_Type::HP_Rigenerazione_Terreno_Naturale as u64 }
            "(caotico)" => { HP_Options_Type::HP_Rigenerazione_Caos as u64 }
            "(effetti e incantesimi caotici)" => { HP_Options_Type::HP_Rigenerazione_Caos as u64 }
            "(caos o magia)" => { HP_Options_Type::HP_Rigenerazione_Caos_Magia as u64 }
            "(bene)" => { HP_Options_Type::HP_Rigenerazione_Bene as u64 }
            "(armi e incantesimi buoni)" => { HP_Options_Type::HP_Rigenerazione_Bene as u64 }
            "(incantesimi buoni, armi buone)" => { HP_Options_Type::HP_Rigenerazione_Bene as u64 }
            "(armi e incantesimi allineati con il bene)" => { HP_Options_Type::HP_Rigenerazione_Bene as u64 }
            "(incantesimi ed armi con descrittore bene)" => { HP_Options_Type::HP_Rigenerazione_Bene as u64 }
            "(armi e incantesimi con il descrittore bene)" => { HP_Options_Type::HP_Rigenerazione_Bene as u64 }
            "(armi ed incantesimi con descrittore bene)" => { HP_Options_Type::HP_Rigenerazione_Bene as u64 }
            "(artefatti, effetti e incantesimi buoni)" => { HP_Options_Type::HP_Rigenerazione_Bene as u64 }
            "(armi o incantesimi con descrittore bene, armi d'argento)" =>
            { HP_Options_Type::HP_Rigenerazione_Bene_Argento as u64 }
            "(armi e incantesimi con descrittore bene, armi d'argento)" =>
            { HP_Options_Type::HP_Rigenerazione_Bene_Argento as u64 }
            "(armi ed incantesimi con descrittore bene, armi d'argento)" =>
            { HP_Options_Type::HP_Rigenerazione_Bene_Argento as u64 }
            "(armi buone, armi d'argento, incantesimi buoni)" =>
            { HP_Options_Type::HP_Rigenerazione_Bene_Argento as u64 }
            "(fuoco o incantesimi buoni)" => { HP_Options_Type::HP_Rigenerazione_Fuoco_Bene as u64 }
            "(argento, armi e incantesimi buoni)" => { HP_Options_Type::HP_Rigenerazione_Bene_Argento as u64 }
            "(armi e incantesimi buoni, ruggine)" => { HP_Options_Type::HP_Rigenerazione_Bene_Ruggine as u64 }
            "(armi e incantesimi con il descrittore male)" => { HP_Options_Type::HP_Rigenerazione_Male as u64 }
            "(armi ed effetti con il descrittore male)" => { HP_Options_Type::HP_Rigenerazione_Male as u64 }
            "(armi, artefatti, effetti ed incantesimi con il descrittore male)" =>
            { HP_Options_Type::HP_Rigenerazione_Male as u64 }
            "(artefatti, effetti e incantesimi malvagi)" => { HP_Options_Type::HP_Rigenerazione_Male as u64 }
            "(effetti e armi malvagie)" => { HP_Options_Type::HP_Rigenerazione_Male as u64 }
            "(armi ed effetti malvagi)" => { HP_Options_Type::HP_Rigenerazione_Male as u64 }
            "(armi o effetti malvagi)" => { HP_Options_Type::HP_Rigenerazione_Male as u64 }
            "(armi e incantesimi malvagi)" => { HP_Options_Type::HP_Rigenerazione_Male as u64 }
            "(male)" => { HP_Options_Type::HP_Rigenerazione_Male as u64 }
            "(legale)" => { HP_Options_Type::HP_Rigenerazione_Legale as u64 }
            "(armi mitiche o incantesimi)" => { HP_Options_Type::HP_Rigenerazione_Mitico as u64 }
            "(deifico o mitico)" => { HP_Options_Type::HP_Rigenerazione_Deifico_Mitico as u64 }
            "(deifica o mitica)" => { HP_Options_Type::HP_Rigenerazione_Deifico_Mitico as u64 }
            "(epico)" => { HP_Options_Type::HP_Rigenerazione_Epico as u64 }
            "(bene, epico o mitico)" => { HP_Options_Type::HP_Rigenerazione_Bene_Epico_Mitico as u64 }
            "(bene, deifico o mitico)" => { HP_Options_Type::HP_Rigenerazione_Bene_Deifico_Mitico as u64 }
            "(freddo)" => { HP_Options_Type::HP_Rigenerazione_Freddo as u64 }
            "(freddo o male)" => { HP_Options_Type::HP_Rigenerazione_Freddo_Male as u64 }
            "(Ferro Freddo)" => { HP_Options_Type::HP_Rigenerazione_Ferro_Freddo as u64 }
            "(ferro freddo)" => { HP_Options_Type::HP_Rigenerazione_Ferro_Freddo as u64 }
            "(acido o fuoco)" => { HP_Options_Type::HP_Rigenerazione_Acido_Fuoco as u64 }
            "(acido e fuoco)" => { HP_Options_Type::HP_Rigenerazione_Acido_Fuoco as u64 }
            "(Fuoco o Acido)" => { HP_Options_Type::HP_Rigenerazione_Acido_Fuoco as u64 }
            "(acido, fuoco)" => { HP_Options_Type::HP_Rigenerazione_Acido_Fuoco as u64 }
            "(acido o fuoco; vedi Vigore Primordiale)" => { HP_Options_Type::HP_Rigenerazione_Acido_Fuoco as u64 }
            "(acido o fuoco, vedi Vigore Primordiale)" => { HP_Options_Type::HP_Rigenerazione_Acido_Fuoco as u64 }
            "(acido o fuoco; solo a contatto con l'acqua)" =>
            { HP_Options_Type::HP_Rigenerazione_Acido_Fuoco_In_Acqua as u64 }
            "(acido)" => { HP_Options_Type::HP_Rigenerazione_Acido as u64 }
            "(acido da incantesimo o oggetto mitico)" => { HP_Options_Type::HP_Rigenerazione_Acido_Mitico as u64 }
            "(acido o sonoro)" => { HP_Options_Type::HP_Rigenerazione_Acido_Sonoro as u64 }
            "(acido, sonoro)" => { HP_Options_Type::HP_Rigenerazione_Acido_Sonoro as u64 }
            "(acido o sonoro, vedi Vigore Primordiale)" => { HP_Options_Type::HP_Rigenerazione_Acido_Sonoro as u64 }
            "(acido o freddo)" => { HP_Options_Type::HP_Rigenerazione_Acido_Freddo as u64 }
            "(acido, freddo o fuoco)" => { HP_Options_Type::HP_Rigenerazione_Acido_Freddo_Fuoco as u64 }
            "(contundente o fuoco)" => { HP_Options_Type::HP_Rigenerazione_Contundente_Fuoco as u64 }
            "(energia positiva)" => { HP_Options_Type::HP_Rigenerazione_Energia_Positiva as u64 }
            "(energia positiva o ispirazione)" =>
            { HP_Options_Type::HP_Rigenerazione_Energia_Positiva_Ispirazione as u64 }
            "(energia negativa)" => { HP_Options_Type::HP_Rigenerazione_Energia_Negativa as u64 }
            "(Pietrificazione)" => { HP_Options_Type::HP_Rigenerazione_Pietrificazione as u64 }
            "(Adamantio)" => { HP_Options_Type::HP_Rigenerazione_Adamantio as u64 }
            "(armi maledette)" => { HP_Options_Type::HP_Rigenerazione_Armi_Maledette as u64 }
            "(armi epiche)" => { HP_Options_Type::HP_Rigenerazione_Armi_Epiche as u64 }
            "(colpo senz'armi o armi naturali)" => { HP_Options_Type::HP_Rigenerazione_No_Armi as u64 }
            "(terra; vedi Rovina della Terra)" => { HP_Options_Type::HP_Rigenerazione_Terra as u64 }
            "(aria; vedi Anatema dell'Aria)" => { HP_Options_Type::HP_Rigenerazione_Aria as u64 }
            "(nella statuetta)" => { HP_Options_Type::HP_Rigenerazione_Statuetta as u64 }
            "(artefatti malvagi)" => { HP_Options_Type::HP_Rigenerazione_Artefatti_Malvagi as u64 }
            "(speciale)" => { HP_Options_Type::HP_Rigenerazione_Speciale as u64 }
            
            _ => { 
                if real_field.len() < 3 { HP_Options_Type::HP_Rigenerazione as u64 }
                else { todo!("Unhandled Rigenerazione Type from {field} in {page_name}"); }
            }
        };
        
        final_value |= (option_type << HP_OPTION_TYPE_OFFSET);
        final_value |= (option_val << HP_OPTION_VAL_OFFSET);
        
        return final_value;
    }
    
    todo!("More stuff left at the end of: {field} in {page_name}");
}