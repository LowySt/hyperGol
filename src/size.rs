pub fn normalize_size<'a>(field: &'a str, page: &str) -> &'a str
{
    match(field)
    {
        "Piccolissimo" => { return "Piccolissima"; }
        "Piccolissima" => { return "Piccolissima"; }
        
        "Minuto" => { return "Minuta"; }
        "Minuta" => { return "Minuta"; }
        
        "Minuscolo" => { return "Minuscola"; }
        "Minuscola" => { return "Minuscola"; }
        
        "Piccolo" => { return "Piccola"; }
        "Piccola" => { return "Piccola"; }
        
        "Medio" => { return "Media"; }
        "Media" => { return "Media"; }
        
        "Grande" => { return "Grande"; }
        
        //NOTE: Privilegio di Classe Iracondo di Stirpe
        "Grande (Mole Demoniaca)" => { return "Grande (Mole Demoniaca)"; }
        
        "Enorme" => { return "Enorme"; }
        
        "Mastodontico" => { return "Mastodontica"; }
        "Mastodontica" => { return "Mastodontica"; }
        
        "Colossale" => { return "Colossale"; }
        
        _ => { todo!("Unhandled size: {field} in {page}"); }
    }
}