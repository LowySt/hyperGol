use crate::entries::*;

//NOTE: Immunities are represented by a 8bit value mapping
//      A single 64bit word contains 8 packed immunities.

const IMM_SENTINEL_VALUE: u64   = 0xFFFFFFFFFFFFFFFF;
const IMM_SINGLE_BIT_LEN: usize = 8;
const IMM_MAX_COUNT:      usize = 8;
const IMM_SINGLE_MAX_VAL: u64   = 255;

#[derive(Debug)]
#[repr(u64)]
#[derive(PartialEq)]
#[derive(Eq)]
#[derive(Clone)]
#[derive(Copy)]
#[derive(PartialOrd)]
#[derive(Ord)]
pub enum Immunity_Type
{
    Immunity_Acid, Immunity_Fire, Immunity_Cold, Immunity_Elec, Immunity_Sound,
    Immunity_EnergyNegative, Immunity_EnergyPositive, Immunity_InfernalFire,
    Immunity_AcidAndFire, Immunity_FireAndCold, Immunity_ElecAndFire, Immunity_ColdAndElec,
    
    Immunity_Paralysis, Immunity_Sleep, Immunity_Fatigued, Immunity_Exhausted, 
    Immunity_Fear, Immunity_Sickened, Immunity_Nauseated, Immunity_Petrified,
    Immunity_Blind, Immunity_Stun, Immunity_Confusion, Immunity_Dazed,
    Immunity_Staggered, Immunity_Dazzled, Immunity_Deaf, Immunity_BlindAndDeaf,
    Immunity_DazzledAndPetrified, Immunity_Shaken, Immunity_Frightened,
    
    Immunity_Poison, Immunity_PoisonIngested, Immunity_Disease, Immunity_Magic,
    Immunity_Criticals, Immunity_SneakAtk, Immunity_GazeAtk, Immunity_WeaponDmg,
    Immunity_Curse, Immunity_Bleeding, Immunity_Bludgeoning, Immunity_FallAndCrush,
    Immunity_SmellAtk, Immunity_Precision, Immunity_PiercingAndSlashing, Immunity_BludgeoningAndPiercing,
    Immunity_Flanking, Immunity_NonLethalDamage, Immunity_Warpproof, Immunity_Piercing, Immunity_Slashing,
    Immunity_Divination, Immunity_PermanentWounds, Immunity_DiseaseAndPoison, Immunity_FatiguedAndExhausted,
    Immunity_DazedAndStun, Immunity_ParalysisAndPetrified, Immunity_BleedingAndPermanentWounds,
    Immunity_SleepAndStun, Immunity_PrecisionAndCriticals, Immunity_BloodDrainAndBleeding,
    Immunity_BloodDrainAndBleedingAndPoison, Immunity_Pressure,
    
    Immunity_MindEffects, Immunity_DeathEffects, Immunity_EmotionEffects, Immunity_GazeEffects,
    Immunity_ForceEffects, Immunity_GazeAndIllusionEffects, Immunity_CharmEffects,
    Immunity_CompulsionEffect, Immunity_MetamorphosisEffects, Immunity_MindRegressionEffects,
    Immunity_CreatureCountEffect, Immunity_MindEffectsAndGaze, Immunity_CompletionAndActivationSpellEffects,
    Immunity_SensesEffects, Immunity_FearAndMindEffects, Immunity_IllusionEffects, 
    Immunity_PainEffects, Immunity_ConfusionCrazyEffects, Immunity_AgingEffects,
    Immunity_ConfusionAndRegressionEffects, Immunity_Possession, Immunity_Rust, Immunity_IllusionSightEffects,
    Immunity_LightEffects, Immunity_CharmAndCompulsionEffect, Immunity_NecromancyEffects,
    Immunity_DeathAndAgingEffects, Immunity_NecromancyAndDeathEffects,
    
    Immunity_CharmeAndIllusionSpells, Immunity_RecallSpells, Immunity_DetectThoughtSpell,
    Immunity_DetectAlignmentSpell, Immunity_DetectLiesSpell, Immunity_SpellImmunitySpell,
    Immunity_DimensionalAnchorSpell, Immunity_CharmeMonsterSpell, Immunity_LightningBoltSpell,
    Immunity_FireballSpell, Immunity_SleepSpell,
    
    Immunity_ASDrain, Immunity_ASDmg, Immunity_LevelDrain, Immunity_ASDmgAndDrain,
    Immunity_LevelDrainAndASDmg, Immunity_LevelDrainAndASDmgAndDrain,
    
    Immunity_UndeadTraits, Immunity_OozeTraits, Immunity_SwarmTraits, Immunity_ConstructTraits,
    Immunity_PlantTraits, Immunity_ElementalTraits, Immunity_DragonTraits,
    
    Immunity_MagicMissile, Immunity_StyxWater, Immunity_CalmEmotions, Immunity_Convocation,
    Immunity_RuneMastery, Immunity_MagicControl, Immunity_Slowness, Immunity_TurnUndead,
    Immunity_EnergySpells, Immunity_Annihilation, Immunity_PlaneShift, Immunity_ChannelEnergyNonMithic,
    Immunity_Labyrinth, Immunity_MemoryLoss, Immunity_BloodDrain, Immunity_LethalGas,
    
    Immunity_ProtectFromEnergy,
    
    //NOTE: This way, when all immunities are invalid, the entire packed value is invalid and viceversa
    Immunity_Invalid = 0xFFFFFFFFFFFFFFFF
}

fn replace_if_present(imm: &mut Vec<Immunity_Type>, to_replace: Immunity_Type, replaced: Immunity_Type,
                      new_count: &mut usize, i: &mut usize)
{
    if let Some(idx) = imm.iter().position(|&x| x == to_replace)
    { 
        let mut to_remove = idx;
        let mut to_keep = *i;
        if idx < *i { to_remove = *i; to_keep = idx}
        
        imm.swap_remove(to_remove);
        imm[to_keep] = replaced;
        *new_count -= 1;
        *i = 0;
    }
}

fn merge_immunities(imm: &mut Vec<Immunity_Type>) -> usize
{
    let mut new_count = imm.len();
    
    let mut keep_running = true;
    let mut i = 0;
    while(keep_running == true)
    {
        match imm[i]
        {
            Immunity_Type::Immunity_Disease => { 
                replace_if_present(imm, Immunity_Type::Immunity_Poison,
                                   Immunity_Type::Immunity_DiseaseAndPoison,
                                   &mut new_count, &mut i);
            }
            
            Immunity_Type::Immunity_Fatigued => {
                replace_if_present(imm, Immunity_Type::Immunity_Exhausted,
                                   Immunity_Type::Immunity_FatiguedAndExhausted,
                                   &mut new_count, &mut i);
            }
            
            Immunity_Type::Immunity_Dazed => {
                replace_if_present(imm, Immunity_Type::Immunity_Stun,
                                   Immunity_Type::Immunity_DazedAndStun,
                                   &mut new_count, &mut i);
            }
            
            Immunity_Type::Immunity_Paralysis => {
                replace_if_present(imm, Immunity_Type::Immunity_Petrified,
                                   Immunity_Type::Immunity_ParalysisAndPetrified,
                                   &mut new_count, &mut i);
            }
            
            Immunity_Type::Immunity_Acid => { 
                replace_if_present(imm, Immunity_Type::Immunity_Fire,
                                   Immunity_Type::Immunity_AcidAndFire,
                                   &mut new_count, &mut i);
            }
            
            Immunity_Type::Immunity_Fire => {
                replace_if_present(imm, Immunity_Type::Immunity_Cold,
                                   Immunity_Type::Immunity_FireAndCold,
                                   &mut new_count, &mut i);
            }
            
            Immunity_Type::Immunity_Cold => {
                replace_if_present(imm, Immunity_Type::Immunity_Elec,
                                   Immunity_Type::Immunity_ColdAndElec,
                                   &mut new_count, &mut i);
            }
            
            
            Immunity_Type::Immunity_Elec => {
                replace_if_present(imm, Immunity_Type::Immunity_Fire,
                                   Immunity_Type::Immunity_ElecAndFire,
                                   &mut new_count, &mut i);
            }
            
            Immunity_Type::Immunity_LevelDrain => { 
                replace_if_present(imm, Immunity_Type::Immunity_ASDmg,
                                   Immunity_Type::Immunity_LevelDrainAndASDmg,
                                   &mut new_count, &mut i);
            }
            
            Immunity_Type::Immunity_Bleeding => { 
                replace_if_present(imm, Immunity_Type::Immunity_PermanentWounds,
                                   Immunity_Type::Immunity_BleedingAndPermanentWounds,
                                   &mut new_count, &mut i);
            }
            
            Immunity_Type::Immunity_CharmEffects => { 
                replace_if_present(imm, Immunity_Type::Immunity_CompulsionEffect,
                                   Immunity_Type::Immunity_CharmAndCompulsionEffect,
                                   &mut new_count, &mut i);
            }
            
            Immunity_Type::Immunity_DeathEffects => { 
                replace_if_present(imm, Immunity_Type::Immunity_AgingEffects,
                                   Immunity_Type::Immunity_DeathAndAgingEffects,
                                   &mut new_count, &mut i);
            }
            
            Immunity_Type::Immunity_LevelDrainAndASDmg => { 
                replace_if_present(imm, Immunity_Type::Immunity_ASDrain,
                                   Immunity_Type::Immunity_LevelDrainAndASDmgAndDrain,
                                   &mut new_count, &mut i);
            }
            
            Immunity_Type::Immunity_Sleep => { 
                replace_if_present(imm, Immunity_Type::Immunity_Stun,
                                   Immunity_Type::Immunity_SleepAndStun,
                                   &mut new_count, &mut i);
            }
            
            Immunity_Type::Immunity_NecromancyEffects => { 
                replace_if_present(imm, Immunity_Type::Immunity_DeathEffects,
                                   Immunity_Type::Immunity_NecromancyAndDeathEffects,
                                   &mut new_count, &mut i);
            }
            
            Immunity_Type::Immunity_Blind => { 
                replace_if_present(imm, Immunity_Type::Immunity_Deaf,
                                   Immunity_Type::Immunity_BlindAndDeaf,
                                   &mut new_count, &mut i);
            }
            
            Immunity_Type::Immunity_Precision => {
                replace_if_present(imm, Immunity_Type::Immunity_Criticals,
                                   Immunity_Type::Immunity_PrecisionAndCriticals,
                                   &mut new_count, &mut i);
            }
            
            Immunity_Type::Immunity_Dazzled => {
                replace_if_present(imm, Immunity_Type::Immunity_Petrified,
                                   Immunity_Type::Immunity_DazzledAndPetrified,
                                   &mut new_count, &mut i);
            }
            
            Immunity_Type::Immunity_BloodDrain => {
                replace_if_present(imm, Immunity_Type::Immunity_Bleeding,
                                   Immunity_Type::Immunity_BloodDrainAndBleeding,
                                   &mut new_count, &mut i);
            }
            
            Immunity_Type::Immunity_BloodDrainAndBleeding => {
                replace_if_present(imm, Immunity_Type::Immunity_Poison,
                                   Immunity_Type::Immunity_BloodDrainAndBleedingAndPoison,
                                   &mut new_count, &mut i);
            }
            
            _ => { }
        }
        
        i += 1;
        if i >= new_count { break; }
    }
    
    return new_count;
}

fn parse_single_immunity(block: &str, page_name: &str) -> Immunity_Type
{
    let mut result: Immunity_Type = Immunity_Type::Immunity_Invalid;
    
    match(block)
    {
        "Acido" | "acido" => { return Immunity_Type::Immunity_Acid; }
        "Fuoco" | "fuoco" | "Assorbimento del Fuoco" => { return Immunity_Type::Immunity_Fire; }
        "Freddo" | "freddo" | "Immunità al Freddo" | "Freddo (sé stesso e chi lo cavalca)"
            => { return Immunity_Type::Immunity_Cold; }
        "Elettricità" | "elettricità" | "Elettricità (parziale)" => { return Immunity_Type::Immunity_Elec; }
        "Sonoro" | "sonoro" => { return Immunity_Type::Immunity_Sound; }
        "Energia Negativa" | "energia negativa" => { return Immunity_Type::Immunity_EnergyNegative; }
        "Energia Positiva" | "energia negativa" => { return Immunity_Type::Immunity_EnergyPositive; }
        "Fuoco Infernale" => { return Immunity_Type::Immunity_InfernalFire; }
        
        "Paralisi" | "paralisi" => { return Immunity_Type::Immunity_Paralysis; }
        "Sonno" | "sonno" | "effetti di sonno" | "Effetti di Sonno" | "Effetti di sonno" |
            "effetti di Sonno"
            => { return Immunity_Type::Immunity_Sleep; }
        "Condizione Affaticato" | "Affaticato" | "Affaticamento" | "Fatica"
            => { return Immunity_Type::Immunity_Fatigued; }
        "Condizione Esausto" | "Esausto" | "Esaurimento" => { return Immunity_Type::Immunity_Exhausted; }
        "Paura" | "paura" | "Effetti di Paura" | "effetti di Paura" | "effetti di paura" |
            "Effetti di paura"
            => { return Immunity_Type::Immunity_Fear; }
        "Infermità" | "Infermo" => { return Immunity_Type::Immunity_Sickened; }
        "Nauseato" | "Nausea" | "nausea" => { return Immunity_Type::Immunity_Nauseated; }
        "Pietrificazione" | "pietrificazione" => { return Immunity_Type::Immunity_Petrified; }
        "Cecità" | "Accecato" =>  { return Immunity_Type::Immunity_Blind; }
        "Stordimento" | "stordimento" => { return Immunity_Type::Immunity_Stun; }
        "Confusione" => { return Immunity_Type::Immunity_Confusion; }
        "Frastornamento" | "Frastornato" => { return Immunity_Type::Immunity_Dazed; }
        "Barcollante" | "effetti Barcollanti" | "Barcollare" => { return Immunity_Type::Immunity_Staggered; }
        "Abbagliato" | "Abbagliamenti" | "Abbagliamento" => { return Immunity_Type::Immunity_Dazzled; }
        "Sordità" | "sordità" => { return Immunity_Type::Immunity_Deaf; }
        "Scosso" => { return Immunity_Type::Immunity_Shaken; }
        "Spaventato" => { return Immunity_Type::Immunity_Frightened; }
        
        "Veleni" | "Veleno" | "veleno" | "veleni" => { return Immunity_Type::Immunity_Poison; }
        "Veleno Ingerito" => { return Immunity_Type::Immunity_PoisonIngested; }
        "Malattie" | "Malattia" | "malattie" | "malattia" => { return Immunity_Type::Immunity_Disease; }
        "Magia" | "Immunità alla Magia" => { return Immunity_Type::Immunity_Magic; }
        "Colpi Critici" | "Colpi critici" => { return Immunity_Type::Immunity_Criticals; }
        "danno di precisione" | "danni da precisione" | "danni di precisione"
            => { return Immunity_Type::Immunity_Precision; }
        "Attacchi Furtivi" => { return Immunity_Type::Immunity_SneakAtk; }
        "Attacchi con Sguardo" | "Attacchi con lo sguardo" | "attacchi con lo Sguardo" |
            "Attacchi dipendenti dalla vista" | "attacchi con lo sguardo" | "attacchi con Sguardo" | 
            "Sguardo" | "Attacchi basati sulla vista" | "Attacchi basati sulla vista o sguardo" =>
        { return Immunity_Type::Immunity_GazeAtk; }
        "Attacchi basati sull'olfatto" => { return Immunity_Type::Immunity_SmellAtk; }
        "Danno da arma" | "danno da arma" | "Danni da arma" | "danni da arma" | "danni delle armi" | 
            "Danni da armi" | "danni da armi" | "Danni dalle Armi" =>
        { return Immunity_Type::Immunity_WeaponDmg; }
        "Armi contundenti" | "Danni contundenti" | "contundente" | "danni contundenti" =>
        { return Immunity_Type::Immunity_Bludgeoning; }
        "danni contundenti e perforanti" => { return Immunity_Type::Immunity_BludgeoningAndPiercing; }
        "Danni perforanti e taglienti" | "danni taglienti e perforanti" =>
        { return Immunity_Type::Immunity_PiercingAndSlashing; }
        "danni perforanti" => { return Immunity_Type::Immunity_Piercing; }
        "Armi Taglienti" | "danni taglienti" => { return Immunity_Type::Immunity_Slashing; }
        "da caduta e da schiacciamento" => { return Immunity_Type::Immunity_FallAndCrush; }
        "Maledizioni" | "maledizioni" | "effetti di maledizione" | "effetti di maledizioni"
            => { return Immunity_Type::Immunity_Curse; }
        "Sanguinamento" | "sanguinamento" | "Sanguinamento (solo Punti Ferita)"
            => { return Immunity_Type::Immunity_Bleeding; }
        "Fiancheggiamento" | "Attacchi ai Fianchi" => { return Immunity_Type::Immunity_Flanking; }
        "Danni Non Letali" | "Danno non letale" => { return Immunity_Type::Immunity_NonLethalDamage; }
        "Non Deformante" => { return Immunity_Type::Immunity_Warpproof; }
        "Danno da Pressione" => { return Immunity_Type::Immunity_Pressure; }
        "possessione" =>  { return Immunity_Type::Immunity_Possession; }
        "ruggine" => { return Immunity_Type::Immunity_Rust; }
        "Divinazione" | "divinazione" => { return Immunity_Type::Immunity_Divination; }
        "ferite permanenti" => { return Immunity_Type::Immunity_PermanentWounds; }
        
        "Effetti di Influenza Mentale" | "Effetti di influenza mentale" | "effetti di influenza mentale" | 
            "Effetti d'influenza mentale" | "controllo mentale" =>
        { return Immunity_Type::Immunity_MindEffects; }
        "effetti di Paura ed influenza mentale" =>
        { return Immunity_Type::Immunity_FearAndMindEffects; }
        "effetti di influenza mentale e visivi" | "di influenza mentale e basati sullo Sguardo" =>
        { return Immunity_Type::Immunity_MindEffectsAndGaze; }
        "Effetti di charme" | "effetti di charme" | "Charme" | "charme" | "Ammaliamento" | "effetti di ammaliamento"
            => { return Immunity_Type::Immunity_CharmEffects; }
        "effetti di compulsione" | "compulsione" =>
        { return Immunity_Type::Immunity_CompulsionEffect; }
        "effetti di charme e compulsioni" | "Effetti di charme e compulsioni" | "Effetti di Charme e Compulsione" |
            "effetti di Charme e Compulsione" | "effetti di charme e compulsione"
            => { return Immunity_Type::Immunity_CharmAndCompulsionEffect; }
        "Effetti di morte" | "Effetti di Morte" | "effetti di morte" | "Effetti di morte (vedi sotto)" =>
        { return Immunity_Type::Immunity_DeathEffects; }
        "illusioni" | "Illusione" => { return Immunity_Type::Immunity_IllusionEffects; }
        "illusioni visive" => { return Immunity_Type::Immunity_IllusionSightEffects; }
        "effetti basati sulla vista" | "effetti visivi" | "attacchi basati sulla vista ed effetti visivi" |
            "Effetti basati sulla vista" | "effetti di Sguardo" | "Effetti visivi" | "effetti di sguardo"
            => { return Immunity_Type::Immunity_GazeEffects; }
        "effetti visivi e illusioni" | "visivi ed illusori" | "effetti e illusioni visive" | 
            "Illusioni e qualsiasi attacco basato sulla vista"
            => { return Immunity_Type::Immunity_GazeAndIllusionEffects; }
        "effetti di metamorfosi" | "Metamorfosi" | "metamorfosi" | "effetti polimorfi" | "effetti di Metamorfosi"
            => { return Immunity_Type::Immunity_MetamorphosisEffects; }
        "Effetti di Demenza" | "effetti di demenza" | "Demenza"
            => { return Immunity_Type::Immunity_MindRegressionEffects; }
        "effetti che bersagliano uno specifico numero di creature" =>
        { return Immunity_Type::Immunity_CreatureCountEffect; }
        "Effetti a completamento di incantesimo e ad attivazione di incantesimo" =>
        { return Immunity_Type::Immunity_CompletionAndActivationSpellEffects; }
        "effetti sensoriali" =>
        { return Immunity_Type::Immunity_SensesEffects; }
        "emozioni" => { return Immunity_Type::Immunity_EmotionEffects; }
        "Effetti di forza" | "forza" | "Forza" => { return Immunity_Type::Immunity_ForceEffects; }
        "Effetti di dolore" | "dolore" | "Dolore" => { return Immunity_Type::Immunity_PainEffects; }
        "Effetti di confusione e follia" => { return Immunity_Type::Immunity_ConfusionCrazyEffects; }
        "effetti di confusione e demenza" => { return Immunity_Type::Immunity_ConfusionAndRegressionEffects; }
        "Invecchiamento" | "invecchiamento" => { return Immunity_Type::Immunity_AgingEffects; }
        "effetti di luce" => { return Immunity_Type::Immunity_LightEffects; }
        "effetti di necromanzia" => { return Immunity_Type::Immunity_NecromancyEffects; }
        
        "Incantesimi di Ammaliamento e di Illusione" => { return Immunity_Type::Immunity_CharmeAndIllusionSpells; }
        "Richiamo" | "Congedo" | "Immunità all'Esilio" => { return Immunity_Type::Immunity_RecallSpells; }
        "Individuazione dei Pensieri" => { return Immunity_Type::Immunity_DetectThoughtSpell; }
        "Individuazione dell'Allineamento" => { return Immunity_Type::Immunity_DetectAlignmentSpell; }
        "Rivela Bugie" => { return Immunity_Type::Immunity_DetectLiesSpell; }
        "Immunità agli Incantesimi" => { return Immunity_Type::Immunity_SpellImmunitySpell; }
        "Ancora Dimensionale" => { return Immunity_Type::Immunity_DimensionalAnchorSpell; }
        "charme sui mostri" => { return Immunity_Type::Immunity_CharmeMonsterSpell; }
        "fulmine" => { return Immunity_Type::Immunity_LightningBoltSpell; }
        "palla di fuoco" => { return Immunity_Type::Immunity_FireballSpell; }
        "sonno" => { return Immunity_Type::Immunity_SleepSpell; }
        
        "Risucchio di Energia" | "risucchio di energia" | "Risucchi di Energia" | "risucchi di energia" |
            "livelli negativi" | "Livelli Negativi"
            => { return Immunity_Type::Immunity_LevelDrain; }
        "Risucchio di Caratteristiche" | "risucchio di caratteristiche" | "risucchi di caratteristiche" |
            "risucchio di caratteristica"
            => { return Immunity_Type::Immunity_ASDrain; }
        "Danni alle caratteristiche" | "Danno alle Caratteristiche" | "danni alle caratteristiche" |
            "danni alle Caratteristiche"
            => { return Immunity_Type::Immunity_ASDmg; }
        "Danni e risucchio di caratteristiche" | "Danni e risucchi di caratteristiche" | 
            "Danno e risucchio alle caratteristiche" | "Danni e risucchio alle Caratteristiche" |
            "Danni e risucchi alle caratteristiche" | "danni e risucchi di Caratteristiche"
            => { return Immunity_Type::Immunity_ASDmgAndDrain; }
        
        "Tratti dei Non Morti" => { return Immunity_Type::Immunity_UndeadTraits; }
        "Tratti delle Melme" => { return Immunity_Type::Immunity_OozeTraits; }
        "Tratti degli Sciami" => { return Immunity_Type::Immunity_SwarmTraits; }
        "Tratti dei Costrutti" => { return Immunity_Type::Immunity_ConstructTraits; }
        "Tratti dei Vegetali" => { return Immunity_Type::Immunity_PlantTraits; }
        "Tratti degli Elementali" => { return Immunity_Type::Immunity_ElementalTraits; }
        "Tratti dei Draghi" | "tratti dei Draghi" => { return Immunity_Type::Immunity_DragonTraits; }
        
        //NOTE: Random fucking shit
        "Dardo Incantato" => { return Immunity_Type::Immunity_MagicMissile; }
        "Calmare Emozioni" => { return Immunity_Type::Immunity_CalmEmotions; }
        "Convocazione" => { return Immunity_Type::Immunity_Convocation; }
        "Immunità alle acque del Fiume Stige" | "immunità alle acque del Fiume Stige" | 
            "acque del Fiume Stige" =>
        { return Immunity_Type::Immunity_StyxWater; }
        "Padronanza delle Rune" => { return Immunity_Type::Immunity_RuneMastery; }
        "Controllo Magico" => { return Immunity_Type::Immunity_MagicControl; }
        "Lentezza" => { return Immunity_Type::Immunity_Slowness; }
        "scacciare" => { return Immunity_Type::Immunity_TurnUndead; }
        "Incantesimi di Energia" => { return Immunity_Type::Immunity_EnergySpells; }
        "Immunità all'Annichilazione" => { return Immunity_Type::Immunity_Annihilation; }
        "Spostamento Planare" => { return Immunity_Type::Immunity_PlaneShift; }
        "Incalare Energia da Fonti Non Mitiche" => { return Immunity_Type::Immunity_ChannelEnergyNonMithic; }
        "Labirinto" =>  { return Immunity_Type::Immunity_Labyrinth; }
        "Immunità alla Perdita di Memoria" => { return Immunity_Type::Immunity_MemoryLoss; }
        "Risucchi di Sangue" => { return Immunity_Type::Immunity_BloodDrain; }
        "Gas e vapori letali" | "vapori e gas letali" => { return Immunity_Type::Immunity_LethalGas; }
        
        //NOTE: This just shouldn't be a fucking immunity... Probably gonna decide what to do in PCMan
        "Fuoco (60 punti)" | "Fuoco (96 punti)" | "Fuoco (108 punti)" | "Fuoco (120 punti)" | 
            "fuoco (120 punti)" | "Freddo (120 punti)" | "Elettricità (120 punti)"
            => { return Immunity_Type::Immunity_ProtectFromEnergy; }
        
        _ => { todo!("Unhandled immunity type: {block} in {page_name}"); }
    }
    
    return result;
}

pub fn prepare_and_map_immunities(defense: &mut Vec<&str>, page_name: &str) -> u64
{
    if defense[IMM_IDX_IN_ARR] == ""  { return IMM_SENTINEL_VALUE; }
    if defense[IMM_IDX_IN_ARR] == "-" { return IMM_SENTINEL_VALUE; }
    
    //NOTE: We remove all prefixes and suffixes that match the below check (because of golarion syntax errors)
    let trim_check: &[_] = &[' ', ',', ';', '.'];
    let mut base = defense[IMM_IDX_IN_ARR].trim_matches(trim_check);
    
    let mut partial_result: Vec<Immunity_Type> = Vec::with_capacity(16);
    
    let mut imm_start_idx = 0;
    let mut char_iter = base.char_indices().peekable();
    while let Some((base_idx, char)) = char_iter.next()
    {
        match char
        {
            //NOTE: We found the end of an immunity
            ',' => {
                //NOTE: We know this is safe because imm_star_idx and base_idx are validly aligned indices
                //      and `base_idx + 1` has to be valid because we just matched an ASCII ',' character.
                let single_imm_block = &base[imm_start_idx..base_idx].trim();
                imm_start_idx = base_idx + 1;
                
                let single_imm: Immunity_Type = parse_single_immunity(single_imm_block, page_name);
                assert!((single_imm as u64) <= IMM_SINGLE_MAX_VAL, "Single Immunity larger than max val");
                
                partial_result.push(single_imm);
            }
            
            //NOTE: Identifiers
            _ => {}
        }
    }
    
    //NOTE: Outside the while we should've hit the end of the string, so the last resistance has to be added
    let remainder = &base[imm_start_idx..].trim();
    if remainder.len() > 2
    {
        let single_imm: Immunity_Type = parse_single_immunity(remainder, page_name);
        assert!((single_imm as u64) <= IMM_SINGLE_MAX_VAL, "Single Immunity larger than max val");
        
        partial_result.push(single_imm);
    }
    
    let mut result = 0u64;
    
    if partial_result.len() <= IMM_MAX_COUNT
    {
        //NOTE: They fit in 64bits, so we just pack and send them!
        for i in 0..partial_result.len()
        {
            result |= ((partial_result[i] as u64) << (i*IMM_SINGLE_BIT_LEN));
        }
        
        for i in partial_result.len()..IMM_MAX_COUNT
        {
            result |= ((0x00000000000000FF as u64) << (i*IMM_SINGLE_BIT_LEN));
        }
        
        return result;
    }
    else
    {
        partial_result.sort();
        partial_result.dedup();
        
        let effective_count = merge_immunities(&mut partial_result);
        
        assert!(effective_count <= IMM_MAX_COUNT, 
                "[{page_name}]\nStill too many immunities: {base}\n[{}] {partial_result:#?}", partial_result.len());
        assert!(partial_result.len() <= IMM_MAX_COUNT, "Effective count and result len are not aligned?");
        
        //NOTE: They fit in 64bits, so we just pack and send them!
        for i in 0..partial_result.len()
        {
            result |= ((partial_result[i] as u64) << (i*IMM_SINGLE_BIT_LEN));
        }
        
        for i in partial_result.len()..IMM_MAX_COUNT
        {
            result |= ((0x00000000000000FF as u64) << (i*IMM_SINGLE_BIT_LEN));
        }
        
        return result;
    }
    
    return result;
}