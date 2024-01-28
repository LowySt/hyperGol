use crate::pages::*;
use crate::entries::*;
use crate::parse_util::*;
use crate::vec_cache::*;

#[derive(Debug)]
pub enum Skill_Names
{
    Acrobazia,
    Saltare,
    Saltare2,
    
    Addestrare,
    Artigianato,
    
    Alchimia,
    Calligrafia,
    Carpenteria,
    Ceramica,
    CostruireArchi,
    CostruireNavi,
    CostruireTrappole,
    CostruireTrappole2,
    FabbricareArmature,
    FabbricareArmi,
    Ferramenta,
    IntessereCeste,
    LavorarePellami,
    LavoriInMuratura,
    Metallurgia,
    Pittura,
    Oreficeria,
    RilegareLibri,
    RiparareScarpe,
    Scultura,
    Tessitura,
    
    Artista,
    Camuffare,
    Cavalcare,
    Conoscenze,
    
    Conoscenze_Arcane,
    Conoscenze_Arcane2,
    Conoscenze_Dungeon,
    Conoscenze_Geografia,
    Conoscenze_Ingegneria,
    Conoscenze_Locali,
    Conoscenze_Locali2,
    Conoscenze_Natura,
    Conoscenze_Nobilt,
    Conoscenze_Piani,
    Conoscenze_Religioni,
    Conoscenze_Religioni2,
    Conoscenze_Storia,
    
    Diplomazia,
    Disattivare,
    Furtivit,
    Guarire,
    Intimidire,
    
    Intrattenere,
    
    Canto,
    Cantare,
    Commedia,
    Danza,
    Oratoria,
    Recitazione,
    Strumenti_Corda,
    Strumenti_Fiato,
    Strumenti_Percussione,
    Strumenti_Tastiera,
    
    Intuizione,
    Linguistica,
    Nuotare,
    Percezione,
    
    Professione,
    
    Allevatore,
    Architetto,
    Avvocato,
    Barcaiolo,
    Bibliotecario,
    Birraio,
    Boscaiolo,
    Cacciatore,
    Carovaniere,
    Conciatore,
    Conestabile,
    Contabile,
    Contadino,
    Cortigiano,
    Cuoco,
    Erborista,
    Farmacista,
    Fornaio,
    Giardiniere,
    Giocatore_Azzardo,
    Guida,
    Ingegnere,
    Levatrice,
    Locandiere,
    Macellaio,
    Marinaio,
    Mercante,
    Minatore,
    Mugnaio,
    Pastore,
    Pescatore,
    Scrivano,
    Siniscalco,
    Soldato,
    Stalliere,
    Taglialegna,
    
    Raggirare,
    Rapidit,
    Sapienza,
    Scalare,
    Sopravvivenza,
    
    SeguireTracce,
    
    Utilizzare,
    Valutare,
    Volare,
    
    Acqua,
    Foreste,
    Paludi,
    
    Ascoltare,
    Individuare,
    
    Tutte,
    Qualsiasi,
    UnaQualsiasi,
    UnoQualsiasi,
}

const SKILLS_BITS: usize = 7;

impl Skill_Names {
    pub fn as_str(&self) -> &'static str
    {
        match self
        {
            Skill_Names::Acrobazia => "Acrobazia",
            Skill_Names::Saltare => "saltare",
            Skill_Names::Saltare2 => "salta",
            
            Skill_Names::Addestrare => "Addestrare Animali",
            Skill_Names::Artigianato => "Artigianato",
            
            Skill_Names::Alchimia => "alchimia",
            Skill_Names::Calligrafia => "calligrafia",
            Skill_Names::Carpenteria => "carpenteria",
            Skill_Names::Ceramica => "ceramica",
            Skill_Names::CostruireArchi => "costruire archi",
            Skill_Names::CostruireNavi => "costruire navi",
            Skill_Names::CostruireTrappole => "costruire trappole",
            Skill_Names::CostruireTrappole2 => "trappole",
            Skill_Names::FabbricareArmature => "fabbricare armature",
            Skill_Names::FabbricareArmi => "fabbricare armi",
            Skill_Names::Ferramenta => "ferramenta",
            Skill_Names::IntessereCeste => "intessere ceste",
            Skill_Names::LavorarePellami => "lavorare pellami",
            Skill_Names::LavoriInMuratura => "lavori in muratura",
            Skill_Names::Metallurgia => "metallurgia",
            Skill_Names::Pittura => "pittura",
            Skill_Names::Oreficeria => "oreficeria",
            Skill_Names::RilegareLibri => "rilegare libri",
            Skill_Names::RiparareScarpe => "riparare scarpe",
            Skill_Names::Scultura => "scultura",
            Skill_Names::Tessitura => "tessitura",
            
            Skill_Names::Artista => "Artista della Fuga",
            Skill_Names::Camuffare => "Camuffare",
            Skill_Names::Cavalcare => "Cavalcare",
            Skill_Names::Conoscenze => "Conoscenze",
            
            Skill_Names::Conoscenze_Arcane => "arcane",
            Skill_Names::Conoscenze_Arcane2 => "arcana",
            Skill_Names::Conoscenze_Dungeon => "dungeon",
            Skill_Names::Conoscenze_Geografia => "geografia",
            Skill_Names::Conoscenze_Ingegneria => "ingegneria",
            Skill_Names::Conoscenze_Locali => "locali",
            Skill_Names::Conoscenze_Locali2 => "locale",
            Skill_Names::Conoscenze_Natura => "natura",
            Skill_Names::Conoscenze_Nobilt => "nobilt\u{00e0}",
            Skill_Names::Conoscenze_Piani => "piani",
            Skill_Names::Conoscenze_Religioni => "religioni",
            Skill_Names::Conoscenze_Religioni2 => "religione",
            Skill_Names::Conoscenze_Storia => "storia",
            
            Skill_Names::Diplomazia => "Diplomazia",
            Skill_Names::Disattivare => "Disattivare Congegni",
            Skill_Names::Furtivit => "Furtivit\u{00e0}",
            Skill_Names::Guarire => "Guarire",
            Skill_Names::Intimidire => "Intimidire",
            Skill_Names::Intrattenere => "Intrattenere",
            
            Skill_Names::Canto => "canto",
            Skill_Names::Cantare => "cantare",
            Skill_Names::Commedia => "commedia",
            Skill_Names::Danza => "danza",
            Skill_Names::Oratoria => "oratoria",
            Skill_Names::Recitazione => "recitazione",
            Skill_Names::Strumenti_Corda => "strumenti a corda",
            Skill_Names::Strumenti_Fiato => "strumenti a fiato",
            Skill_Names::Strumenti_Percussione => "strumenti a percussione",
            Skill_Names::Strumenti_Tastiera => "strumenti a tastiera",
            
            Skill_Names::Intuizione => "Intuizione",
            Skill_Names::Linguistica => "Linguistica",
            Skill_Names::Nuotare => "Nuotare",
            Skill_Names::Percezione => "Percezione",
            Skill_Names::Professione => "Professione",
            
            Skill_Names::Allevatore => "allevatore",
            Skill_Names::Architetto => "architetto",
            Skill_Names::Avvocato => "avvocato",
            Skill_Names::Barcaiolo => "barcaiolo",
            Skill_Names::Bibliotecario => "bibliotecario",
            Skill_Names::Birraio => "birraio",
            Skill_Names::Boscaiolo => "boscaiolo",
            Skill_Names::Cacciatore => "cacciatore",
            Skill_Names::Carovaniere => "carovaniere",
            Skill_Names::Conciatore => "conciatore",
            Skill_Names::Conestabile => "conestabile",
            Skill_Names::Contabile => "contabile",
            Skill_Names::Contadino => "contadino",
            Skill_Names::Cortigiano => "cortigiano",
            Skill_Names::Cuoco => "cuoco",
            Skill_Names::Erborista => "erborista",
            Skill_Names::Farmacista => "farmacista",
            Skill_Names::Fornaio => "fornaio",
            Skill_Names::Giardiniere => "giardiniere",
            Skill_Names::Giocatore_Azzardo => "giocatore d'azzardo",
            Skill_Names::Guida => "guida",
            Skill_Names::Ingegnere => "ingegnere",
            Skill_Names::Levatrice => "levatrice",
            Skill_Names::Locandiere => "locandiere",
            Skill_Names::Macellaio => "macellaio",
            Skill_Names::Marinaio => "marinaio",
            Skill_Names::Mercante => "mercante",
            Skill_Names::Minatore => "minatore",
            Skill_Names::Mugnaio => "mugnaio",
            Skill_Names::Pastore => "pastore",
            Skill_Names::Pescatore => "pescatore",
            Skill_Names::Scrivano => "scrivano",
            Skill_Names::Siniscalco => "siniscalco",
            Skill_Names::Soldato => "soldato",
            Skill_Names::Stalliere => "stalliere",
            Skill_Names::Taglialegna => "taglialegna",
            
            Skill_Names::Raggirare => "Raggirare",
            Skill_Names::Rapidit => "Rapidit\u{00e0} di Mano",
            Skill_Names::Sapienza => "Sapienza Magica",
            Skill_Names::Scalare => "Scalare",
            Skill_Names::Sopravvivenza => "Sopravvivenza",
            
            Skill_Names::SeguireTracce => "seguire tracce",
            
            Skill_Names::Utilizzare => "Utilizzare Congegni Magici",
            Skill_Names::Valutare => "Valutare",
            Skill_Names::Volare => "Volare",
            
            Skill_Names::Acqua => "acqua",
            Skill_Names::Foreste => "foreste",
            Skill_Names::Paludi => "paludi",
            
            Skill_Names::Ascoltare => "Ascoltare",
            Skill_Names::Individuare => "Individuare",
            
            Skill_Names::Tutte => "tutte",
            Skill_Names::Qualsiasi => "qualsiasi",
            Skill_Names::UnaQualsiasi => "una qualsiasi",
            Skill_Names::UnoQualsiasi => "uno qualsiasi",
        }
    }
    
    pub fn from_str(name: &str) -> u16
    {
        match name
        {
            "Acrobazia" => Skill_Names::Acrobazia as u16,
            "saltare" => Skill_Names::Saltare as u16,
            "salta" => Skill_Names::Saltare2 as u16,
            
            "Addestrare Animali" => Skill_Names::Addestrare as u16,
            "Artigianato" => Skill_Names::Artigianato as u16,
            
            "alchimia" => Skill_Names::Alchimia as u16,
            "calligrafia" => Skill_Names::Calligrafia as u16,
            "carpenteria" => Skill_Names::Carpenteria as u16,
            "ceramica" => Skill_Names::Ceramica as u16,
            "costruire archi" => Skill_Names::CostruireArchi as u16,
            "costruire navi" => Skill_Names::CostruireNavi as u16,
            "costruire trappole" => Skill_Names::CostruireTrappole as u16,
            "trappole" => Skill_Names::CostruireTrappole2 as u16,
            "fabbricare armature" => Skill_Names::FabbricareArmature as u16,
            "fabbricare armi" => Skill_Names::FabbricareArmi as u16,
            "ferramenta" => Skill_Names::Ferramenta as u16,
            "intessere ceste" => Skill_Names::IntessereCeste as u16,
            "lavorare pellami" => Skill_Names::LavorarePellami as u16,
            "lavori in muratura" => Skill_Names::LavoriInMuratura as u16,
            "metallurgia" => Skill_Names::Metallurgia as u16,
            "pittura" => Skill_Names::Pittura as u16,
            "oreficeria" => Skill_Names::Oreficeria as u16,
            "rilegare libri" => Skill_Names::RilegareLibri as u16,
            "riparare scarpe" => Skill_Names::RiparareScarpe as u16,
            "scultura" => Skill_Names::Scultura as u16,
            "tessitura" => Skill_Names::Tessitura as u16,
            
            "Artista della Fuga" => Skill_Names::Artista as u16,
            "Camuffare" => Skill_Names::Camuffare as u16,
            "Cavalcare" => Skill_Names::Cavalcare as u16,
            
            "Conoscenze" => Skill_Names::Conoscenze as u16,
            "arcane" => Skill_Names::Conoscenze_Arcane as u16,
            "arcana" => Skill_Names::Conoscenze_Arcane2 as u16,
            "dungeon" => Skill_Names::Conoscenze_Dungeon as u16,
            "geografia" => Skill_Names::Conoscenze_Geografia as u16,
            "ingegneria" => Skill_Names::Conoscenze_Ingegneria as u16,
            "locali" => Skill_Names::Conoscenze_Locali as u16,
            "locale" => Skill_Names::Conoscenze_Locali as u16,
            "natura" => Skill_Names::Conoscenze_Natura as u16,
            "nobilt\u{00e0}" => Skill_Names::Conoscenze_Nobilt as u16,
            "piani" => Skill_Names::Conoscenze_Piani as u16,
            "religioni" => Skill_Names::Conoscenze_Religioni as u16,
            "religione" => Skill_Names::Conoscenze_Religioni2 as u16,
            "storia" => Skill_Names::Conoscenze_Storia as u16,
            
            "Diplomazia" => Skill_Names::Diplomazia as u16,
            "Disattivare Congegni" => Skill_Names::Disattivare as u16,
            "Furtivit\u{00e0}" => Skill_Names::Furtivit as u16,
            "Guarire" => Skill_Names::Guarire as u16,
            "Intimidire" => Skill_Names::Intimidire as u16,
            "Intrattenere" => Skill_Names::Intrattenere as u16,
            
            "canto" => Skill_Names::Canto as u16,
            "cantare" => Skill_Names::Cantare as u16,
            "commedia" => Skill_Names::Commedia as u16,
            "danza" => Skill_Names::Danza as u16,
            "oratoria" => Skill_Names::Oratoria as u16,
            "recitazione" => Skill_Names::Recitazione as u16,
            "strumenti a corda" => Skill_Names::Strumenti_Corda as u16,
            "strumenti a fiato" => Skill_Names::Strumenti_Fiato as u16,
            "strumenti a percussione" => Skill_Names::Strumenti_Percussione as u16,
            "strumenti a tastiera" => Skill_Names::Strumenti_Tastiera as u16,
            
            "Intuizione" => Skill_Names::Intuizione as u16,
            "Linguistica" => Skill_Names::Linguistica as u16,
            "Nuotare" => Skill_Names::Nuotare as u16,
            "Percezione" => Skill_Names::Percezione as u16,
            "Professione" => Skill_Names::Professione as u16,
            
            "allevatore" => Skill_Names::Allevatore as u16,
            "architetto" => Skill_Names::Architetto as u16,
            "avvocato" => Skill_Names::Avvocato as u16,
            "barcaiolo" => Skill_Names::Barcaiolo as u16,
            "bibliotecario" => Skill_Names::Bibliotecario as u16,
            "birraio" => Skill_Names::Birraio as u16,
            "boscaiolo" => Skill_Names::Boscaiolo as u16,
            "cacciatore" => Skill_Names::Cacciatore as u16,
            "carovaniere" => Skill_Names::Carovaniere as u16,
            "conciatore" => Skill_Names::Conciatore as u16,
            "conestabile" => Skill_Names::Conestabile as u16,
            "contabile" => Skill_Names::Contabile as u16,
            "contadino" => Skill_Names::Contadino as u16,
            "cortigiano" => Skill_Names::Cortigiano as u16,
            "cuoco" => Skill_Names::Cuoco as u16,
            "erborista" => Skill_Names::Erborista as u16,
            "farmacista" => Skill_Names::Farmacista as u16,
            "fornaio" => Skill_Names::Fornaio as u16,
            "giardiniere" => Skill_Names::Giardiniere as u16,
            "giocatore d'azzardo" => Skill_Names::Giocatore_Azzardo as u16,
            "guida" => Skill_Names::Guida as u16,
            "ingegnere" => Skill_Names::Ingegnere as u16,
            "levatrice" => Skill_Names::Levatrice as u16,
            "locandiere" => Skill_Names::Locandiere as u16,
            "macellaio" => Skill_Names::Macellaio as u16,
            "marinaio" => Skill_Names::Marinaio as u16,
            "mercante" => Skill_Names::Mercante as u16,
            "minatore" => Skill_Names::Minatore as u16,
            "mugnaio" => Skill_Names::Mugnaio as u16,
            "pastore" => Skill_Names::Pastore as u16,
            "pescatore" => Skill_Names::Pescatore as u16,
            "scrivano" => Skill_Names::Scrivano as u16,  //TODO: Fuck golarion. "scriba"
            "siniscalco" => Skill_Names::Siniscalco as u16,
            "soldato" => Skill_Names::Soldato as u16,
            "stalliere" => Skill_Names::Stalliere as u16,
            "taglialegna" => Skill_Names::Taglialegna as u16,
            
            "Raggirare" => Skill_Names::Raggirare as u16,
            "Rapidit\u{00e0} di Mano" => Skill_Names::Rapidit as u16,
            "Sapienza Magica" => Skill_Names::Sapienza as u16,
            "Scalare" => Skill_Names::Scalare as u16,
            "Sopravvivenza" => Skill_Names::Sopravvivenza as u16,
            
            "seguire tracce" => Skill_Names::SeguireTracce as u16,
            
            "Utilizzare Congegni Magici" => Skill_Names::Utilizzare as u16,
            "Valutare" => Skill_Names::Valutare as u16,
            "Volare" => Skill_Names::Volare as u16,
            
            "acqua" => Skill_Names::Acqua as u16,
            "foreste" => Skill_Names::Foreste as u16,
            "paludi" => Skill_Names::Paludi as u16,
            
            "Ascoltare" => Skill_Names::Ascoltare as u16,
            "Individuare" => Skill_Names::Individuare as u16,
            
            "tutte" => Skill_Names::Tutte as u16,
            "qualsiasi" => Skill_Names::Qualsiasi as u16,
            "una qualsiasi" => Skill_Names::UnaQualsiasi as u16,
            "uno qualsiasi" => Skill_Names::UnoQualsiasi as u16,
            
            _ => 0xffff,
        }
    }
    
    pub fn is_valid(name: &str) -> bool
    {
        match name
        {
            "Acrobazia" => true,
            "saltare" => true,
            "salta" => true,
            
            "Addestrare Animali" => true,
            "Artigianato" => true,
            
            "alchimia" => true,
            "calligrafia" => true,
            "carpenteria" => true,
            "ceramica" => true,
            "costruire archi" => true,
            "costruire navi" => true,
            "costruire trappole" => true,
            "trappole" => true,
            "fabbricare armature" => true,
            "fabbricare armi" => true,
            "ferramenta" => true,
            "intessere ceste" => true,
            "lavorare pellami" => true,
            "lavori in muratura" => true,
            "metallurgia" => true,
            "pittura" => true,
            "oreficeria" => true,
            "rilegare libri" => true,
            "riparare scarpe" => true,
            "scultura" => true,
            "tessitura" => true,
            
            "Artista della Fuga" => true,
            "Camuffare" => true,
            "Cavalcare" => true,
            
            "Conoscenze" => true,
            "arcane" => true,
            "arcana" => true,
            "dungeon" => true,
            "geografia" => true,
            "ingegneria" => true,
            "locali" => true,
            "locale" => true,
            "natura" => true,
            "nobilt\u{00e0}" => true,
            "piani" => true,
            "religioni" => true,
            "religione" => true,
            "storia" => true,
            
            "Diplomazia" => true,
            "Disattivare Congegni" => true,
            "Furtivit\u{00e0}" => true,
            "Guarire" => true,
            "Intimidire" => true,
            "Intrattenere" => true,
            
            "canto" => true,
            "cantare" => true,
            "commedia" => true,
            "danza" => true,
            "oratoria" => true,
            "recitazione" => true,
            "strumenti a corda" => true,
            "strumenti a fiato" => true,
            "strumenti a percussione" => true,
            "strumenti a tastiera" => true,
            
            "Intuizione" => true,
            "Linguistica" => true,
            "Nuotare" => true,
            "Percezione" => true,
            "Professione" => true,
            
            "allevatore" => true,
            "architetto" => true,
            "avvocato" => true,
            "barcaiolo" => true,
            "bibliotecario" => true,
            "birraio" => true,
            "boscaiolo" => true,
            "cacciatore" => true,
            "carovaniere" => true,
            "conciatore" => true,
            "conestabile" => true,
            "contabile" => true,
            "contadino" => true,
            "cortigiano" => true,
            "cuoco" => true,
            "erborista" => true,
            "farmacista" => true,
            "fornaio" => true,
            "giardiniere" => true,
            "giocatore d'azzardo" => true,
            "guida" => true,
            "ingegnere" => true,
            "levatrice" => true,
            "locandiere" => true,
            "macellaio" => true,
            "marinaio" => true,
            "mercante" => true,
            "minatore" => true,
            "mugnaio" => true,
            "pastore" => true,
            "pescatore" => true,
            "scrivano" => true,
            "siniscalco" => true,
            "soldato" => true,
            "stalliere" => true,
            "taglialegna" => true,
            
            "Raggirare" => true,
            "Rapidit\u{00e0} di Mano" => true,
            "Sapienza Magica" => true,
            "Scalare" => true,
            "Sopravvivenza" => true,
            
            "seguire tracce" => true,
            
            "Utilizzare Congegni Magici" => true,
            "Valutare" => true,
            "Volare" => true,
            
            "acqua" => true,
            "foreste" => true,
            "paludi" => true,
            
            "Ascoltare" => true,
            "Individuare" => true,
            
            "tutte" => true,
            "qualsiasi" => true,
            "una qualsiasi" => true,
            "uno qualsiasi" => true,
            
            _ => false,
        }
    }
    
    pub fn from_u16(v: u16) -> Skill_Names
    {
        match v
        {
            n if v == Skill_Names::Acrobazia as u16 => Skill_Names::Acrobazia,
            n if v == Skill_Names::Saltare as u16 => Skill_Names::Saltare,
            n if v == Skill_Names::Saltare2 as u16 => Skill_Names::Saltare2,
            
            n if v == Skill_Names::Addestrare as u16 => Skill_Names::Addestrare,
            n if v == Skill_Names::Artigianato as u16 => Skill_Names::Artigianato,
            n if v == Skill_Names::Alchimia as u16 => Skill_Names::Alchimia,
            n if v == Skill_Names::Calligrafia as u16 => Skill_Names::Calligrafia,
            n if v == Skill_Names::Carpenteria as u16 => Skill_Names::Carpenteria,
            n if v == Skill_Names::Ceramica as u16 => Skill_Names::Ceramica,
            n if v == Skill_Names::CostruireArchi as u16 => Skill_Names::CostruireArchi,
            n if v == Skill_Names::CostruireNavi as u16 => Skill_Names::CostruireNavi,
            n if v == Skill_Names::CostruireTrappole as u16 => Skill_Names::CostruireTrappole,
            n if v == Skill_Names::CostruireTrappole2 as u16 => Skill_Names::CostruireTrappole2,
            n if v == Skill_Names::FabbricareArmature as u16 => Skill_Names::FabbricareArmature,
            n if v == Skill_Names::FabbricareArmi as u16 => Skill_Names::FabbricareArmi,
            n if v == Skill_Names::Ferramenta as u16 => Skill_Names::Ferramenta,
            n if v == Skill_Names::IntessereCeste as u16 => Skill_Names::IntessereCeste,
            n if v == Skill_Names::LavorarePellami as u16 => Skill_Names::LavorarePellami,
            n if v == Skill_Names::LavoriInMuratura as u16 => Skill_Names::LavoriInMuratura,
            n if v == Skill_Names::Metallurgia as u16 => Skill_Names::Metallurgia,
            n if v == Skill_Names::Pittura as u16 => Skill_Names::Pittura,
            n if v == Skill_Names::Oreficeria as u16 => Skill_Names::Oreficeria,
            n if v == Skill_Names::RilegareLibri as u16 => Skill_Names::RilegareLibri,
            n if v == Skill_Names::RiparareScarpe as u16 => Skill_Names::RiparareScarpe,
            n if v == Skill_Names::Scultura as u16 => Skill_Names::Scultura,
            n if v == Skill_Names::Tessitura as u16 => Skill_Names::Tessitura,
            n if v == Skill_Names::Artista as u16 => Skill_Names::Artista,
            n if v == Skill_Names::Camuffare as u16 => Skill_Names::Camuffare,
            n if v == Skill_Names::Cavalcare as u16 => Skill_Names::Cavalcare,
            n if v == Skill_Names::Conoscenze as u16 => Skill_Names::Conoscenze,
            n if v == Skill_Names::Conoscenze_Arcane as u16 => Skill_Names::Conoscenze_Arcane,
            n if v == Skill_Names::Conoscenze_Arcane2 as u16 => Skill_Names::Conoscenze_Arcane2,
            n if v == Skill_Names::Conoscenze_Dungeon as u16 => Skill_Names::Conoscenze_Dungeon,
            n if v == Skill_Names::Conoscenze_Geografia as u16 => Skill_Names::Conoscenze_Geografia,
            n if v == Skill_Names::Conoscenze_Ingegneria as u16 => Skill_Names::Conoscenze_Ingegneria,
            n if v == Skill_Names::Conoscenze_Locali as u16 => Skill_Names::Conoscenze_Locali,
            n if v == Skill_Names::Conoscenze_Locali2 as u16 => Skill_Names::Conoscenze_Locali2,
            n if v == Skill_Names::Conoscenze_Natura as u16 => Skill_Names::Conoscenze_Natura,
            n if v == Skill_Names::Conoscenze_Nobilt as u16 => Skill_Names::Conoscenze_Nobilt,
            n if v == Skill_Names::Conoscenze_Piani as u16 => Skill_Names::Conoscenze_Piani,
            n if v == Skill_Names::Conoscenze_Religioni as u16 => Skill_Names::Conoscenze_Religioni,
            n if v == Skill_Names::Conoscenze_Religioni2 as u16 => Skill_Names::Conoscenze_Religioni2,
            n if v == Skill_Names::Conoscenze_Storia as u16 => Skill_Names::Conoscenze_Storia,
            n if v == Skill_Names::Diplomazia as u16 => Skill_Names::Diplomazia,
            n if v == Skill_Names::Disattivare as u16 => Skill_Names::Disattivare,
            n if v == Skill_Names::Furtivit as u16 => Skill_Names::Furtivit,
            n if v == Skill_Names::Guarire as u16 => Skill_Names::Guarire,
            n if v == Skill_Names::Intimidire as u16 => Skill_Names::Intimidire,
            n if v == Skill_Names::Intrattenere as u16 => Skill_Names::Intrattenere,
            n if v == Skill_Names::Canto as u16 => Skill_Names::Canto,
            n if v == Skill_Names::Cantare as u16 => Skill_Names::Cantare,
            n if v == Skill_Names::Commedia as u16 =>  Skill_Names::Commedia,
            n if v == Skill_Names::Danza as u16 =>  Skill_Names::Danza,
            n if v == Skill_Names::Oratoria as u16 =>  Skill_Names::Oratoria,
            n if v == Skill_Names::Recitazione as u16 =>  Skill_Names::Recitazione,
            n if v == Skill_Names::Strumenti_Corda as u16 =>  Skill_Names::Strumenti_Corda,
            n if v == Skill_Names::Strumenti_Fiato as u16 =>  Skill_Names::Strumenti_Fiato,
            n if v == Skill_Names::Strumenti_Percussione as u16 =>  Skill_Names::Strumenti_Percussione,
            n if v == Skill_Names::Strumenti_Tastiera as u16 =>  Skill_Names::Strumenti_Tastiera,
            n if v == Skill_Names::Intuizione as u16 => Skill_Names::Intuizione,
            n if v == Skill_Names::Linguistica as u16 => Skill_Names::Linguistica,
            n if v == Skill_Names::Nuotare as u16 => Skill_Names::Nuotare,
            n if v == Skill_Names::Percezione as u16 => Skill_Names::Percezione,
            n if v == Skill_Names::Professione as u16 => Skill_Names::Professione,
            n if v == Skill_Names::Allevatore as u16 => Skill_Names::Allevatore,
            n if v == Skill_Names::Architetto as u16 =>  Skill_Names::Architetto,
            n if v == Skill_Names::Avvocato as u16 =>  Skill_Names::Avvocato,
            n if v == Skill_Names::Barcaiolo as u16 =>  Skill_Names::Barcaiolo,
            n if v == Skill_Names::Bibliotecario as u16 =>  Skill_Names::Bibliotecario,
            n if v == Skill_Names::Birraio as u16 =>  Skill_Names::Birraio,
            n if v == Skill_Names::Boscaiolo as u16 =>  Skill_Names::Boscaiolo,
            n if v == Skill_Names::Cacciatore as u16 =>  Skill_Names::Cacciatore,
            n if v == Skill_Names::Carovaniere as u16 =>  Skill_Names::Carovaniere,
            n if v == Skill_Names::Conciatore as u16 =>  Skill_Names::Conciatore,
            n if v == Skill_Names::Conestabile as u16 =>  Skill_Names::Conestabile,
            n if v == Skill_Names::Contabile as u16 =>  Skill_Names::Contabile,
            n if v == Skill_Names::Contadino as u16 =>  Skill_Names::Contadino,
            n if v == Skill_Names::Cortigiano as u16 =>  Skill_Names::Cortigiano,
            n if v == Skill_Names::Cuoco as u16 =>  Skill_Names::Cuoco,
            n if v == Skill_Names::Erborista as u16 =>  Skill_Names::Erborista,
            n if v == Skill_Names::Farmacista as u16 =>  Skill_Names::Farmacista,
            n if v == Skill_Names::Fornaio as u16 =>  Skill_Names::Fornaio,
            n if v == Skill_Names::Giardiniere as u16 =>  Skill_Names::Giardiniere,
            n if v == Skill_Names::Giocatore_Azzardo as u16 =>  Skill_Names::Giocatore_Azzardo,
            n if v == Skill_Names::Guida as u16 =>  Skill_Names::Guida,
            n if v == Skill_Names::Ingegnere as u16 =>  Skill_Names::Ingegnere,
            n if v == Skill_Names::Levatrice as u16 =>  Skill_Names::Levatrice,
            n if v == Skill_Names::Locandiere as u16 =>  Skill_Names::Locandiere,
            n if v == Skill_Names::Macellaio as u16 =>  Skill_Names::Macellaio,
            n if v == Skill_Names::Marinaio as u16 =>  Skill_Names::Marinaio,
            n if v == Skill_Names::Mercante as u16 =>  Skill_Names::Mercante,
            n if v == Skill_Names::Minatore as u16 =>  Skill_Names::Minatore,
            n if v == Skill_Names::Mugnaio as u16 =>  Skill_Names::Mugnaio,
            n if v == Skill_Names::Pastore as u16 =>  Skill_Names::Pastore,
            n if v == Skill_Names::Pescatore as u16 =>  Skill_Names::Pescatore,
            n if v == Skill_Names::Scrivano as u16 =>  Skill_Names::Scrivano,
            n if v == Skill_Names::Siniscalco as u16 =>  Skill_Names::Siniscalco,
            n if v == Skill_Names::Soldato as u16 =>  Skill_Names::Soldato,
            n if v == Skill_Names::Stalliere as u16 =>  Skill_Names::Stalliere,
            n if v == Skill_Names::Taglialegna as u16 =>  Skill_Names::Taglialegna,
            n if v == Skill_Names::Raggirare as u16 => Skill_Names::Raggirare,
            n if v == Skill_Names::Rapidit as u16 => Skill_Names::Rapidit,
            n if v == Skill_Names::Sapienza as u16 => Skill_Names::Sapienza,
            n if v == Skill_Names::Scalare as u16 => Skill_Names::Scalare,
            n if v == Skill_Names::Sopravvivenza as u16 => Skill_Names::Sopravvivenza,
            
            n if v == Skill_Names::SeguireTracce as u16 => Skill_Names::SeguireTracce,
            
            n if v == Skill_Names::Utilizzare as u16 => Skill_Names::Utilizzare,
            n if v == Skill_Names::Valutare as u16 => Skill_Names::Valutare,
            n if v == Skill_Names::Volare as u16 => Skill_Names::Volare,
            
            n if v == Skill_Names::Acqua as u16 => Skill_Names::Acqua,
            n if v == Skill_Names::Foreste as u16 => Skill_Names::Foreste,
            n if v == Skill_Names::Paludi as u16 => Skill_Names::Paludi,
            
            n if v == Skill_Names::Ascoltare as u16 => Skill_Names::Ascoltare,
            n if v == Skill_Names::Individuare as u16 => Skill_Names::Individuare,
            
            n if v == Skill_Names::Tutte as u16 => Skill_Names::Tutte,
            n if v == Skill_Names::Qualsiasi as u16 => Skill_Names::Qualsiasi,
            n if v == Skill_Names::UnaQualsiasi as u16 => Skill_Names::UnaQualsiasi,
            n if v == Skill_Names::UnoQualsiasi as u16 => Skill_Names::UnoQualsiasi,
            _ => panic!("Unhandled Skill_Names from u16"),
        }
    }
}

struct Skill_Field<'a> /*'*/
{
    skill_block: &'a /*'*/ str,
    value_block: &'a /*'*/ str,
    paren_block_whole: &'a /*'*/ str,
    paren_blocks: [&'a /*'*/ str; 16],
    paren_count: usize,
}

//TODO: Not even using value_block. What the fuck.
fn map_or_intern(field: &Skill_Field, cache: &mut VectorCache, bufs: &mut Buffer_Context, page_addr: &str) -> ([u32; 24], usize)
{
    let mut temp_arr = [0u32; 24];
    let mut curr_idx = 1;
    
    let trim_check: &[_] = &["per", "a", "quando", "nelle", "in", "nell'", "sott'"];
    
    //NOTE: First we check if the paren block is mappable. Otherwise we intern
    for i in 0..field.paren_count
    {
        if let Some((skill_value, rest)) = split_at_value(field.paren_blocks[i])
        {
            //NOTE: Block has value
            let to_check = rest.trim();
            let to_check = trim_str(to_check, trim_check).trim().to_lowercase();
            if Skill_Names::is_valid(&to_check)
            {
                let skill_entry = pack_value_u32(Skill_Names::from_str(&to_check) as u32, 
                                                 skill_value as i32, SKILLS_BITS, 0x00FF, PAREN_BIT_U32);
                temp_arr[curr_idx] = skill_entry;
                curr_idx += 1;
            }
            else
            {
                //NOTE: Block has value, but tag is invalid. Intern
                temp_arr[0]  = add_entry_if_missing_u32(&mut cache.skills, &mut bufs.skills, field.skill_block);
                temp_arr[0] |= INTERN_BIT_U32;
                
                return (temp_arr, 1);
            }
        }
        else
        {
            //NOTE: Block either has no value, 
            //      or value is surrounded by text
            let to_check = field.paren_blocks[i].trim().to_lowercase();
            if Skill_Names::is_valid(&to_check)
            {
                let skill_entry = pack_value_u32(Skill_Names::from_str(&to_check) as u32, 
                                                 SENTINEL_VALUE_I32, SKILLS_BITS, 0x00FF, PAREN_BIT_U32);
                temp_arr[curr_idx] = skill_entry;
                curr_idx += 1;
            }
            else
            {
                //NOTE: Block has no value and tag is invalid. Intern.
                temp_arr[0] = add_entry_if_missing_u32(&mut cache.skills, &mut bufs.skills, field.skill_block);
                temp_arr[0] |= INTERN_BIT_U32;
                
                return (temp_arr, 1);
            }
        }
    }
    
    //NOTE: Here we've already checked for parentheses if there's any.
    //      Now only check if the type is valid.
    let without_paren = field.skill_block.replace(field.paren_block_whole, "");
    if let Some((skill_value, rest)) = split_at_value(&without_paren)
    {
        let trimmed = rest.trim();
        if Skill_Names::is_valid(&trimmed)
        {
            //NOTE: Skill has value and is valid.
            let skill_entry = pack_value_u32(Skill_Names::from_str(&trimmed) as u32,
                                             skill_value as i32, SKILLS_BITS, 0x00FF, 0);
            temp_arr[0] = skill_entry;
        }
        else
        {
            //NOTE: Skill has value but is not valid
            temp_arr[0] = add_entry_if_missing_u32(&mut cache.skills, &mut bufs.skills, field.skill_block);
            temp_arr[0] |= INTERN_BIT_U32;
            
            return (temp_arr, 1);
        }
    }
    else
    {
        //NOTE: Skill has no value. If valid we assume it's zero. Otherwise, we intern
        let trimmed = without_paren.trim();
        if Skill_Names::is_valid(&trimmed)
        {
            //NOTE: Skill has no value and is valid.
            let skill_entry = pack_value_u32(Skill_Names::from_str(&trimmed) as u32,
                                             SENTINEL_VALUE_I32, SKILLS_BITS, 0x00FF, 0);
            temp_arr[0] = skill_entry;
        }
        else
        {
            //NOTE: Skill has no value and is not valid
            temp_arr[0] = add_entry_if_missing_u32(&mut cache.skills, &mut bufs.skills, field.skill_block);
            temp_arr[0] |= INTERN_BIT_U32;
            
            return (temp_arr, 1);
        }
    }
    
    return (temp_arr, curr_idx);
}

//TODO: Multiple parentheses handling!! I've only seen 1 case so far, but it's possible!
pub fn prepare_skill_str(stats_arr: &mut Vec<&str>, cache: &mut VectorCache, 
                         bufs: &mut Buffer_Context, page_addr: &str, skill_off: usize) -> [u32; 24]
{
    //NOTE: Early exit when there's no skills.
    if stats_arr[SKILLS_IDX_IN_ARR + skill_off] == "-" { return [0u32; 24]; }
    
    let mut result_arr = [0u32; 24];
    let mut result_idx = 0;
    
    //NOTE: Removing from the array doesn't work, because the elements are precisely placed. So we copy
    let trim_check: &[_] = &[' ', ',', ';', '.'];
    let base = stats_arr[SKILLS_IDX_IN_ARR + skill_off].trim_matches(trim_check);
    
    let mut skill_field = Skill_Field {
        skill_block: "",
        value_block: "",
        paren_block_whole: "",
        paren_blocks: [""; 16],
        paren_count: 0,
    };
    
    let mut skill_start_idx = 0;
    let mut char_iter = base.char_indices().peekable();
    while let iter_opt = char_iter.next()
    {
        match iter_opt
        {
            Some((base_idx, char)) => {
                match char
                {
                    //NOTE: We found the end of this specific skill
                    ',' => {
                        skill_field.skill_block = &base[skill_start_idx..base_idx].trim();
                        skill_start_idx = base_idx + 1;
                        
                        let (temp_arr, count) = map_or_intern(&skill_field, cache, bufs, page_addr);
                        
                        //NOTE: Now we've made sure that the entire skill_block is mappable.
                        //      We can now copy the temp_arr into the result array at the proper index.
                        for (dst, src) in result_arr[result_idx..].iter_mut().zip(&temp_arr[..count]) {
                            *dst = *src;
                            result_idx += 1;
                        }
                        
                        //NOTE: Reset the skill_field for the next skill.
                        skill_field = Skill_Field {
                            skill_block: "",
                            value_block: "",
                            paren_block_whole: "",
                            paren_blocks: [""; 16],
                            paren_count: 0,
                        };
                    }
                    
                    //NOTE: We found a specialization/condition for this skill
                    //      We need to skip until the end of the paren_block
                    //NOTE: Now we traverse this block to divide it into comma separated chunks
                    '(' => {
                        let mut whole_idx = base_idx;
                        let mut start_idx = base_idx;
                        let mut end_idx = 0;
                        let mut depth = 1;
                        
                        while let Some((paren_idx, paren_char)) = char_iter.next()
                        {
                            match paren_char
                            {
                                '(' => { depth += 1; }
                                ')' => {
                                    depth -= 1;
                                    if depth == 0
                                    {
                                        skill_field.paren_block_whole = &base[whole_idx..paren_idx+1];
                                        skill_field.paren_blocks[skill_field.paren_count] = &base[start_idx+1..paren_idx];
                                        skill_field.paren_count += 1;
                                        break;
                                    }
                                }
                                
                                ',' => {
                                    //NOTE: We only subdivide top-level paren block elements.
                                    if depth == 1
                                    {
                                        skill_field.paren_blocks[skill_field.paren_count] = &base[start_idx+1..paren_idx];
                                        start_idx = paren_idx;
                                        skill_field.paren_count += 1;
                                    }
                                }
                                
                                ' ' => {
                                    //NOTE: We check spaces because the separator could be an ' e '
                                    if depth == 1
                                    {
                                        if let Some(e_sep) = base.get(paren_idx..paren_idx+3)
                                        {
                                            if e_sep == " e "
                                            {
                                                skill_field.paren_blocks[skill_field.paren_count] = &base[start_idx+1..paren_idx];
                                                start_idx = paren_idx+2;
                                                skill_field.paren_count += 1;
                                            }
                                        }
                                    }
                                }
                                
                                _ => {}
                            }
                        }
                        
                        assert!(skill_field.paren_count > 0, "{:#?}\n\tCan't fill paren block: {:#?} at {:#?}",
                                page_addr, base, start_idx);
                    }
                    
                    //NOTE: We found the bonus for this skill
                    '+'|'-' => {
                        let start_idx = base_idx;
                        let mut end_idx = start_idx;
                        
                        while let Some((value_idx, value_char)) = char_iter.peek()
                        {
                            if !value_char.is_digit(10) { break; }
                            
                            //NOTE: Consume only if it's still part of the value.
                            end_idx += 1;
                            char_iter.next();
                        }
                        
                        assert!(end_idx != 0, "{:#?}\n\tCan't find end of value block: {:#?} at {:#?}", page_addr, base, start_idx);
                        skill_field.value_block = &base[start_idx..end_idx+1];
                    }
                    
                    //TODO: Add cases for \n handling?
                    
                    //NOTE: Identifiers.
                    _ => {}
                }
            }
            
            None =>
            {
                skill_field.skill_block = &base[skill_start_idx..].trim();
                
                let (temp_arr, count) = map_or_intern(&skill_field, cache, bufs, page_addr);
                
                //NOTE: Now we've made sure that the entire skill_block is mappable.
                //      We can now copy the temp_arr into the result array at the proper index.
                for (dst, src) in result_arr[result_idx..].iter_mut().zip(&temp_arr[..count]) {
                    *dst = *src;
                    result_idx += 1;
                }
                
                break;
            }
        }
    }
    
    return result_arr;
}