// ============================================================================
// imports
// ============================================================================

use std::collections::HashSet;
use std::collections::VecDeque;

use super::Province;
use super::Unit;
use super::UnitKind;

// ============================================================================
// definitions
// ============================================================================

///
/// 経路の構造体
///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Path {
    pub(crate) origin: &'static str,
    pub(crate) dest: &'static str,
    pub(crate) army: bool,
    pub(crate) fleet: bool,
}

/// 経路情報の定数配列
#[rustfmt::skip]
const PATHS: &[Path] = &[
    Path { origin: "adr", dest: "apu", army: false, fleet: true },
    Path { origin: "adr", dest: "ion", army: false, fleet: true },
    Path { origin: "adr", dest: "alb", army: false, fleet: true },
    Path { origin: "adr", dest: "tri", army: false, fleet: true },
    Path { origin: "adr", dest: "ven", army: false, fleet: true },
    Path { origin: "aeg", dest: "gre", army: false, fleet: true },
    Path { origin: "aeg", dest: "smy", army: false, fleet: true },
    Path { origin: "aeg", dest: "bul_sc", army: false, fleet: true },
    Path { origin: "aeg", dest: "eas", army: false, fleet: true },
    Path { origin: "aeg", dest: "con", army: false, fleet: true },
    Path { origin: "aeg", dest: "ion", army: false, fleet: true },
    Path { origin: "alb", dest: "adr", army: false, fleet: true },
    Path { origin: "alb", dest: "gre", army: true, fleet: true },
    Path { origin: "alb", dest: "ser", army: true, fleet: false },
    Path { origin: "alb", dest: "ion", army: false, fleet: true },
    Path { origin: "alb", dest: "tri", army: true, fleet: true },
    Path { origin: "ank", dest: "smy", army: true, fleet: false },
    Path { origin: "ank", dest: "bla", army: false, fleet: true },
    Path { origin: "ank", dest: "arm", army: true, fleet: true },
    Path { origin: "ank", dest: "con", army: true, fleet: true },
    Path { origin: "apu", dest: "rom", army: true, fleet: false },
    Path { origin: "apu", dest: "ion", army: false, fleet: true },
    Path { origin: "apu", dest: "ven", army: true, fleet: true },
    Path { origin: "apu", dest: "adr", army: false, fleet: true },
    Path { origin: "apu", dest: "nap", army: true, fleet: true },
    Path { origin: "arm", dest: "bla", army: false, fleet: true },
    Path { origin: "arm", dest: "syr", army: true, fleet: false },
    Path { origin: "arm", dest: "ank", army: true, fleet: true },
    Path { origin: "arm", dest: "sev", army: true, fleet: true },
    Path { origin: "arm", dest: "smy", army: true, fleet: false },
    Path { origin: "bal", dest: "den", army: false, fleet: true },
    Path { origin: "bal", dest: "ber", army: false, fleet: true },
    Path { origin: "bal", dest: "kie", army: false, fleet: true },
    Path { origin: "bal", dest: "swe", army: false, fleet: true },
    Path { origin: "bal", dest: "lvn", army: false, fleet: true },
    Path { origin: "bal", dest: "bot", army: false, fleet: true },
    Path { origin: "bal", dest: "pru", army: false, fleet: true },
    Path { origin: "bar", dest: "nwy", army: false, fleet: true },
    Path { origin: "bar", dest: "nrg", army: false, fleet: true },
    Path { origin: "bar", dest: "stp_nc", army: false, fleet: true },
    Path { origin: "bel", dest: "eng", army: false, fleet: true },
    Path { origin: "bel", dest: "pic", army: true, fleet: true },
    Path { origin: "bel", dest: "hol", army: true, fleet: true },
    Path { origin: "bel", dest: "ruh", army: true, fleet: false },
    Path { origin: "bel", dest: "nth", army: false, fleet: true },
    Path { origin: "bel", dest: "bur", army: true, fleet: false },
    Path { origin: "ber", dest: "bal", army: false, fleet: true },
    Path { origin: "ber", dest: "kie", army: true, fleet: true },
    Path { origin: "ber", dest: "mun", army: true, fleet: false },
    Path { origin: "ber", dest: "sil", army: true, fleet: false },
    Path { origin: "ber", dest: "pru", army: true, fleet: true },
    Path { origin: "bla", dest: "arm", army: false, fleet: true },
    Path { origin: "bla", dest: "con", army: false, fleet: true },
    Path { origin: "bla", dest: "ank", army: false, fleet: true },
    Path { origin: "bla", dest: "sev", army: false, fleet: true },
    Path { origin: "bla", dest: "rum", army: false, fleet: true },
    Path { origin: "bla", dest: "bul_ec", army: false, fleet: true },
    Path { origin: "boh", dest: "mun", army: true, fleet: false },
    Path { origin: "boh", dest: "sil", army: true, fleet: false },
    Path { origin: "boh", dest: "gal", army: true, fleet: false },
    Path { origin: "boh", dest: "vie", army: true, fleet: false },
    Path { origin: "boh", dest: "tyr", army: true, fleet: false },
    Path { origin: "bot", dest: "swe", army: false, fleet: true },
    Path { origin: "bot", dest: "fin", army: false, fleet: true },
    Path { origin: "bot", dest: "lvn", army: false, fleet: true },
    Path { origin: "bot", dest: "stp_sc", army: false, fleet: true },
    Path { origin: "bot", dest: "bal", army: false, fleet: true },
    Path { origin: "bre", dest: "mid", army: false, fleet: true },
    Path { origin: "bre", dest: "eng", army: false, fleet: true },
    Path { origin: "bre", dest: "par", army: true, fleet: false },
    Path { origin: "bre", dest: "pic", army: true, fleet: true },
    Path { origin: "bre", dest: "gas", army: true, fleet: true },
    Path { origin: "bud", dest: "tri", army: true, fleet: false },
    Path { origin: "bud", dest: "ser", army: true, fleet: false },
    Path { origin: "bud", dest: "gal", army: true, fleet: false },
    Path { origin: "bud", dest: "vie", army: true, fleet: false },
    Path { origin: "bud", dest: "rum", army: true, fleet: false },
    Path { origin: "bul", dest: "con", army: true, fleet: false },
    Path { origin: "bul", dest: "ser", army: true, fleet: false },
    Path { origin: "bul", dest: "gre", army: true, fleet: false },
    Path { origin: "bul", dest: "rum", army: true, fleet: false },
    Path { origin: "bul_ec", dest: "con", army: false, fleet: true },
    Path { origin: "bul_ec", dest: "bla", army: false, fleet: true },
    Path { origin: "bul_ec", dest: "rum", army: false, fleet: true },
    Path { origin: "bul_sc", dest: "con", army: false, fleet: true },
    Path { origin: "bul_sc", dest: "aeg", army: false, fleet: true },
    Path { origin: "bul_sc", dest: "gre", army: false, fleet: true },
    Path { origin: "bur", dest: "par", army: true, fleet: false },
    Path { origin: "bur", dest: "pic", army: true, fleet: false },
    Path { origin: "bur", dest: "gas", army: true, fleet: false },
    Path { origin: "bur", dest: "ruh", army: true, fleet: false },
    Path { origin: "bur", dest: "bel", army: true, fleet: false },
    Path { origin: "bur", dest: "mar", army: true, fleet: false },
    Path { origin: "bur", dest: "mun", army: true, fleet: false },
    Path { origin: "cly", dest: "edi", army: true, fleet: true },
    Path { origin: "cly", dest: "nat", army: false, fleet: true },
    Path { origin: "cly", dest: "lvp", army: true, fleet: true },
    Path { origin: "cly", dest: "nrg", army: false, fleet: true },
    Path { origin: "con", dest: "aeg", army: false, fleet: true },
    Path { origin: "con", dest: "ank", army: true, fleet: true },
    Path { origin: "con", dest: "bla", army: false, fleet: true },
    Path { origin: "con", dest: "bul", army: true, fleet: false },
    Path { origin: "con", dest: "bul_sc", army: false, fleet: true },
    Path { origin: "con", dest: "bul_ec", army: false, fleet: true },
    Path { origin: "con", dest: "smy", army: true, fleet: true },
    Path { origin: "den", dest: "swe", army: true, fleet: true },
    Path { origin: "den", dest: "hel", army: false, fleet: true },
    Path { origin: "den", dest: "bal", army: false, fleet: true },
    Path { origin: "den", dest: "nth", army: false, fleet: true },
    Path { origin: "den", dest: "ska", army: false, fleet: true },
    Path { origin: "den", dest: "kie", army: true, fleet: true },
    Path { origin: "eas", dest: "syr", army: false, fleet: true },
    Path { origin: "eas", dest: "aeg", army: false, fleet: true },
    Path { origin: "eas", dest: "ion", army: false, fleet: true },
    Path { origin: "eas", dest: "smy", army: false, fleet: true },
    Path { origin: "edi", dest: "yor", army: true, fleet: true },
    Path { origin: "edi", dest: "nth", army: false, fleet: true },
    Path { origin: "edi", dest: "nrg", army: false, fleet: true },
    Path { origin: "edi", dest: "cly", army: true, fleet: true },
    Path { origin: "edi", dest: "lvp", army: true, fleet: false },
    Path { origin: "eng", dest: "mid", army: false, fleet: true },
    Path { origin: "eng", dest: "lon", army: false, fleet: true },
    Path { origin: "eng", dest: "nth", army: false, fleet: true },
    Path { origin: "eng", dest: "bel", army: false, fleet: true },
    Path { origin: "eng", dest: "pic", army: false, fleet: true },
    Path { origin: "eng", dest: "bre", army: false, fleet: true },
    Path { origin: "eng", dest: "iri", army: false, fleet: true },
    Path { origin: "eng", dest: "wal", army: false, fleet: true },
    Path { origin: "fin", dest: "bot", army: false, fleet: true },
    Path { origin: "fin", dest: "swe", army: true, fleet: true },
    Path { origin: "fin", dest: "nwy", army: true, fleet: false },
    Path { origin: "fin", dest: "stp_sc", army: false, fleet: true },
    Path { origin: "fin", dest: "stp", army: true, fleet: false },
    Path { origin: "gal", dest: "boh", army: true, fleet: false },
    Path { origin: "gal", dest: "sil", army: true, fleet: false },
    Path { origin: "gal", dest: "rum", army: true, fleet: false },
    Path { origin: "gal", dest: "ukr", army: true, fleet: false },
    Path { origin: "gal", dest: "war", army: true, fleet: false },
    Path { origin: "gal", dest: "vie", army: true, fleet: false },
    Path { origin: "gal", dest: "bud", army: true, fleet: false },
    Path { origin: "gas", dest: "mid", army: false, fleet: true },
    Path { origin: "gas", dest: "spa", army: true, fleet: false },
    Path { origin: "gas", dest: "spa_nc", army: false, fleet: true },
    Path { origin: "gas", dest: "bre", army: true, fleet: true },
    Path { origin: "gas", dest: "bur", army: true, fleet: false },
    Path { origin: "gas", dest: "mar", army: true, fleet: false },
    Path { origin: "gas", dest: "par", army: true, fleet: false },
    Path { origin: "gre", dest: "alb", army: true, fleet: true },
    Path { origin: "gre", dest: "ion", army: false, fleet: true },
    Path { origin: "gre", dest: "ser", army: true, fleet: false },
    Path { origin: "gre", dest: "aeg", army: false, fleet: true },
    Path { origin: "gre", dest: "bul", army: true, fleet: false },
    Path { origin: "gre", dest: "bul_sc", army: false, fleet: true },
    Path { origin: "hel", dest: "den", army: false, fleet: true },
    Path { origin: "hel", dest: "kie", army: false, fleet: true },
    Path { origin: "hel", dest: "nth", army: false, fleet: true },
    Path { origin: "hel", dest: "hol", army: false, fleet: true },
    Path { origin: "hol", dest: "hel", army: false, fleet: true },
    Path { origin: "hol", dest: "bel", army: true, fleet: true },
    Path { origin: "hol", dest: "nth", army: false, fleet: true },
    Path { origin: "hol", dest: "ruh", army: true, fleet: false },
    Path { origin: "hol", dest: "kie", army: true, fleet: true },
    Path { origin: "ion", dest: "nap", army: false, fleet: true },
    Path { origin: "ion", dest: "eas", army: false, fleet: true },
    Path { origin: "ion", dest: "gre", army: false, fleet: true },
    Path { origin: "ion", dest: "apu", army: false, fleet: true },
    Path { origin: "ion", dest: "alb", army: false, fleet: true },
    Path { origin: "ion", dest: "aeg", army: false, fleet: true },
    Path { origin: "ion", dest: "adr", army: false, fleet: true },
    Path { origin: "ion", dest: "tun", army: false, fleet: true },
    Path { origin: "ion", dest: "tyn", army: false, fleet: true },
    Path { origin: "iri", dest: "mid", army: false, fleet: true },
    Path { origin: "iri", dest: "nat", army: false, fleet: true },
    Path { origin: "iri", dest: "lvp", army: false, fleet: true },
    Path { origin: "iri", dest: "eng", army: false, fleet: true },
    Path { origin: "iri", dest: "wal", army: false, fleet: true },
    Path { origin: "kie", dest: "ruh", army: true, fleet: false },
    Path { origin: "kie", dest: "hol", army: true, fleet: true },
    Path { origin: "kie", dest: "mun", army: true, fleet: false },
    Path { origin: "kie", dest: "hel", army: false, fleet: true },
    Path { origin: "kie", dest: "den", army: true, fleet: true },
    Path { origin: "kie", dest: "ber", army: true, fleet: true },
    Path { origin: "kie", dest: "bal", army: false, fleet: true },
    Path { origin: "lon", dest: "nth", army: false, fleet: true },
    Path { origin: "lon", dest: "eng", army: false, fleet: true },
    Path { origin: "lon", dest: "wal", army: true, fleet: true },
    Path { origin: "lon", dest: "yor", army: true, fleet: true },
    Path { origin: "lvn", dest: "bal", army: false, fleet: true },
    Path { origin: "lvn", dest: "stp", army: true, fleet: false },
    Path { origin: "lvn", dest: "stp_sc", army: false, fleet: true },
    Path { origin: "lvn", dest: "mos", army: true, fleet: false },
    Path { origin: "lvn", dest: "bot", army: false, fleet: true },
    Path { origin: "lvn", dest: "pru", army: true, fleet: true },
    Path { origin: "lvn", dest: "war", army: true, fleet: false },
    Path { origin: "lvp", dest: "yor", army: true, fleet: false },
    Path { origin: "lvp", dest: "cly", army: true, fleet: true },
    Path { origin: "lvp", dest: "wal", army: true, fleet: true },
    Path { origin: "lvp", dest: "nat", army: false, fleet: true },
    Path { origin: "lvp", dest: "iri", army: false, fleet: true },
    Path { origin: "lvp", dest: "edi", army: true, fleet: false },
    Path { origin: "gol", dest: "mar", army: false, fleet: true },
    Path { origin: "gol", dest: "spa_sc", army: false, fleet: true },
    Path { origin: "gol", dest: "pie", army: false, fleet: true },
    Path { origin: "gol", dest: "wes", army: false, fleet: true },
    Path { origin: "gol", dest: "tyn", army: false, fleet: true },
    Path { origin: "gol", dest: "tus", army: false, fleet: true },
    Path { origin: "mid", dest: "spa_nc", army: false, fleet: true },
    Path { origin: "mid", dest: "bre", army: false, fleet: true },
    Path { origin: "mid", dest: "por", army: false, fleet: true },
    Path { origin: "mid", dest: "gas", army: false, fleet: true },
    Path { origin: "mid", dest: "nat", army: false, fleet: true },
    Path { origin: "mid", dest: "naf", army: false, fleet: true },
    Path { origin: "mid", dest: "wes", army: false, fleet: true },
    Path { origin: "mid", dest: "eng", army: false, fleet: true },
    Path { origin: "mid", dest: "iri", army: false, fleet: true },
    Path { origin: "mid", dest: "spa_sc", army: false, fleet: true },
    Path { origin: "mar", dest: "spa", army: true, fleet: false },
    Path { origin: "mar", dest: "pie", army: true, fleet: true },
    Path { origin: "mar", dest: "gas", army: true, fleet: false },
    Path { origin: "mar", dest: "spa_sc", army: false, fleet: true },
    Path { origin: "mar", dest: "bur", army: true, fleet: false },
    Path { origin: "mar", dest: "gol", army: false, fleet: true },
    Path { origin: "mos", dest: "lvn", army: true, fleet: false },
    Path { origin: "mos", dest: "ukr", army: true, fleet: false },
    Path { origin: "mos", dest: "stp", army: true, fleet: false },
    Path { origin: "mos", dest: "sev", army: true, fleet: false },
    Path { origin: "mos", dest: "war", army: true, fleet: false },
    Path { origin: "mun", dest: "sil", army: true, fleet: false },
    Path { origin: "mun", dest: "boh", army: true, fleet: false },
    Path { origin: "mun", dest: "kie", army: true, fleet: false },
    Path { origin: "mun", dest: "ber", army: true, fleet: false },
    Path { origin: "mun", dest: "tyr", army: true, fleet: false },
    Path { origin: "mun", dest: "ruh", army: true, fleet: false },
    Path { origin: "mun", dest: "bur", army: true, fleet: false },
    Path { origin: "naf", dest: "tun", army: true, fleet: true },
    Path { origin: "naf", dest: "mid", army: false, fleet: true },
    Path { origin: "naf", dest: "wes", army: false, fleet: true },
    Path { origin: "nat", dest: "lvp", army: false, fleet: true },
    Path { origin: "nat", dest: "mid", army: false, fleet: true },
    Path { origin: "nat", dest: "cly", army: false, fleet: true },
    Path { origin: "nat", dest: "iri", army: false, fleet: true },
    Path { origin: "nat", dest: "nrg", army: false, fleet: true },
    Path { origin: "nap", dest: "ion", army: false, fleet: true },
    Path { origin: "nap", dest: "rom", army: true, fleet: true },
    Path { origin: "nap", dest: "tyn", army: false, fleet: true },
    Path { origin: "nap", dest: "apu", army: true, fleet: true },
    Path { origin: "nth", dest: "yor", army: false, fleet: true },
    Path { origin: "nth", dest: "nwy", army: false, fleet: true },
    Path { origin: "nth", dest: "nrg", army: false, fleet: true },
    Path { origin: "nth", dest: "ska", army: false, fleet: true },
    Path { origin: "nth", dest: "lon", army: false, fleet: true },
    Path { origin: "nth", dest: "bel", army: false, fleet: true },
    Path { origin: "nth", dest: "den", army: false, fleet: true },
    Path { origin: "nth", dest: "edi", army: false, fleet: true },
    Path { origin: "nth", dest: "eng", army: false, fleet: true },
    Path { origin: "nth", dest: "hel", army: false, fleet: true },
    Path { origin: "nth", dest: "hol", army: false, fleet: true },
    Path { origin: "nrg", dest: "edi", army: false, fleet: true },
    Path { origin: "nrg", dest: "cly", army: false, fleet: true },
    Path { origin: "nrg", dest: "nth", army: false, fleet: true },
    Path { origin: "nrg", dest: "bar", army: false, fleet: true },
    Path { origin: "nrg", dest: "nat", army: false, fleet: true },
    Path { origin: "nrg", dest: "nwy", army: false, fleet: true },
    Path { origin: "nwy", dest: "nth", army: false, fleet: true },
    Path { origin: "nwy", dest: "nrg", army: false, fleet: true },
    Path { origin: "nwy", dest: "bar", army: false, fleet: true },
    Path { origin: "nwy", dest: "stp", army: true, fleet: false },
    Path { origin: "nwy", dest: "stp_nc", army: false, fleet: true },
    Path { origin: "nwy", dest: "ska", army: false, fleet: true },
    Path { origin: "nwy", dest: "fin", army: true, fleet: false },
    Path { origin: "nwy", dest: "swe", army: true, fleet: true },
    Path { origin: "par", dest: "bur", army: true, fleet: false },
    Path { origin: "par", dest: "pic", army: true, fleet: false },
    Path { origin: "par", dest: "gas", army: true, fleet: false },
    Path { origin: "par", dest: "bre", army: true, fleet: false },
    Path { origin: "pic", dest: "eng", army: false, fleet: true },
    Path { origin: "pic", dest: "bel", army: true, fleet: true },
    Path { origin: "pic", dest: "bur", army: true, fleet: false },
    Path { origin: "pic", dest: "bre", army: true, fleet: true },
    Path { origin: "pic", dest: "par", army: true, fleet: false },
    Path { origin: "pie", dest: "gol", army: false, fleet: true },
    Path { origin: "pie", dest: "ven", army: true, fleet: false },
    Path { origin: "pie", dest: "mar", army: true, fleet: true },
    Path { origin: "pie", dest: "tyr", army: true, fleet: false },
    Path { origin: "pie", dest: "tus", army: true, fleet: true },
    Path { origin: "por", dest: "spa_nc", army: false, fleet: true },
    Path { origin: "por", dest: "mid", army: false, fleet: true },
    Path { origin: "por", dest: "spa", army: true, fleet: false },
    Path { origin: "por", dest: "spa_sc", army: false, fleet: true },
    Path { origin: "pru", dest: "lvn", army: true, fleet: true },
    Path { origin: "pru", dest: "war", army: true, fleet: false },
    Path { origin: "pru", dest: "ber", army: true, fleet: true },
    Path { origin: "pru", dest: "bal", army: false, fleet: true },
    Path { origin: "pru", dest: "sil", army: true, fleet: false },
    Path { origin: "rom", dest: "ven", army: true, fleet: false },
    Path { origin: "rom", dest: "tus", army: true, fleet: true },
    Path { origin: "rom", dest: "tyn", army: false, fleet: true },
    Path { origin: "rom", dest: "apu", army: true, fleet: false },
    Path { origin: "rom", dest: "nap", army: true, fleet: true },
    Path { origin: "ruh", dest: "hol", army: true, fleet: false },
    Path { origin: "ruh", dest: "mun", army: true, fleet: false },
    Path { origin: "ruh", dest: "kie", army: true, fleet: false },
    Path { origin: "ruh", dest: "bur", army: true, fleet: false },
    Path { origin: "ruh", dest: "bel", army: true, fleet: false },
    Path { origin: "rum", dest: "bud", army: true, fleet: false },
    Path { origin: "rum", dest: "ukr", army: true, fleet: false },
    Path { origin: "rum", dest: "bla", army: false, fleet: true },
    Path { origin: "rum", dest: "gal", army: true, fleet: false },
    Path { origin: "rum", dest: "ser", army: true, fleet: false },
    Path { origin: "rum", dest: "bul", army: true, fleet: false },
    Path { origin: "rum", dest: "bul_ec", army: false, fleet: true },
    Path { origin: "rum", dest: "sev", army: true, fleet: true },
    Path { origin: "ser", dest: "bul", army: true, fleet: false },
    Path { origin: "ser", dest: "gre", army: true, fleet: false },
    Path { origin: "ser", dest: "tri", army: true, fleet: false },
    Path { origin: "ser", dest: "bud", army: true, fleet: false },
    Path { origin: "ser", dest: "alb", army: true, fleet: false },
    Path { origin: "ser", dest: "rum", army: true, fleet: false },
    Path { origin: "sev", dest: "mos", army: true, fleet: false },
    Path { origin: "sev", dest: "rum", army: true, fleet: true },
    Path { origin: "sev", dest: "bla", army: false, fleet: true },
    Path { origin: "sev", dest: "arm", army: true, fleet: true },
    Path { origin: "sev", dest: "ukr", army: true, fleet: false },
    Path { origin: "sil", dest: "war", army: true, fleet: false },
    Path { origin: "sil", dest: "gal", army: true, fleet: false },
    Path { origin: "sil", dest: "pru", army: true, fleet: false },
    Path { origin: "sil", dest: "mun", army: true, fleet: false },
    Path { origin: "sil", dest: "ber", army: true, fleet: false },
    Path { origin: "sil", dest: "boh", army: true, fleet: false },
    Path { origin: "ska", dest: "nth", army: false, fleet: true },
    Path { origin: "ska", dest: "nwy", army: false, fleet: true },
    Path { origin: "ska", dest: "swe", army: false, fleet: true },
    Path { origin: "ska", dest: "den", army: false, fleet: true },
    Path { origin: "smy", dest: "con", army: true, fleet: true },
    Path { origin: "smy", dest: "ank", army: true, fleet: false },
    Path { origin: "smy", dest: "arm", army: true, fleet: false },
    Path { origin: "smy", dest: "syr", army: true, fleet: true },
    Path { origin: "smy", dest: "aeg", army: false, fleet: true },
    Path { origin: "smy", dest: "eas", army: false, fleet: true },
    Path { origin: "spa", dest: "gas", army: true, fleet: false },
    Path { origin: "spa", dest: "mar", army: true, fleet: false },
    Path { origin: "spa", dest: "por", army: true, fleet: false },
    Path { origin: "spa_nc", dest: "gas", army: false, fleet: true },
    Path { origin: "spa_nc", dest: "por", army: false, fleet: true },
    Path { origin: "spa_nc", dest: "mid", army: false, fleet: true },
    Path { origin: "spa_sc", dest: "mar", army: false, fleet: true },
    Path { origin: "spa_sc", dest: "mid", army: false, fleet: true },
    Path { origin: "spa_sc", dest: "gol", army: false, fleet: true },
    Path { origin: "spa_sc", dest: "wes", army: false, fleet: true },
    Path { origin: "spa_sc", dest: "por", army: false, fleet: true },
    Path { origin: "stp", dest: "nwy", army: true, fleet: false },
    Path { origin: "stp", dest: "fin", army: true, fleet: false },
    Path { origin: "stp", dest: "lvn", army: true, fleet: false },
    Path { origin: "stp", dest: "mos", army: true, fleet: false },
    Path { origin: "stp_nc", dest: "bar", army: false, fleet: true },
    Path { origin: "stp_nc", dest: "nwy", army: false, fleet: true },
    Path { origin: "stp_sc", dest: "fin", army: false, fleet: true },
    Path { origin: "stp_sc", dest: "lvn", army: false, fleet: true },
    Path { origin: "stp_sc", dest: "bot", army: false, fleet: true },
    Path { origin: "swe", dest: "ska", army: false, fleet: true },
    Path { origin: "swe", dest: "den", army: true, fleet: true },
    Path { origin: "swe", dest: "nwy", army: true, fleet: true },
    Path { origin: "swe", dest: "fin", army: true, fleet: true },
    Path { origin: "swe", dest: "bot", army: false, fleet: true },
    Path { origin: "swe", dest: "bal", army: false, fleet: true },
    Path { origin: "syr", dest: "arm", army: true, fleet: false },
    Path { origin: "syr", dest: "eas", army: false, fleet: true },
    Path { origin: "syr", dest: "smy", army: true, fleet: true },
    Path { origin: "tri", dest: "vie", army: true, fleet: false },
    Path { origin: "tri", dest: "alb", army: true, fleet: true },
    Path { origin: "tri", dest: "adr", army: false, fleet: true },
    Path { origin: "tri", dest: "ven", army: true, fleet: true },
    Path { origin: "tri", dest: "ser", army: true, fleet: false },
    Path { origin: "tri", dest: "tyr", army: true, fleet: false },
    Path { origin: "tri", dest: "bud", army: true, fleet: false },
    Path { origin: "tun", dest: "wes", army: false, fleet: true },
    Path { origin: "tun", dest: "tyn", army: false, fleet: true },
    Path { origin: "tun", dest: "ion", army: false, fleet: true },
    Path { origin: "tun", dest: "naf", army: true, fleet: true },
    Path { origin: "tus", dest: "gol", army: false, fleet: true },
    Path { origin: "tus", dest: "ven", army: true, fleet: false },
    Path { origin: "tus", dest: "tyn", army: false, fleet: true },
    Path { origin: "tus", dest: "pie", army: true, fleet: true },
    Path { origin: "tus", dest: "rom", army: true, fleet: true },
    Path { origin: "tyr", dest: "vie", army: true, fleet: false },
    Path { origin: "tyr", dest: "tri", army: true, fleet: false },
    Path { origin: "tyr", dest: "mun", army: true, fleet: false },
    Path { origin: "tyr", dest: "boh", army: true, fleet: false },
    Path { origin: "tyr", dest: "ven", army: true, fleet: false },
    Path { origin: "tyr", dest: "pie", army: true, fleet: false },
    Path { origin: "tyn", dest: "tus", army: false, fleet: true },
    Path { origin: "tyn", dest: "ion", army: false, fleet: true },
    Path { origin: "tyn", dest: "rom", army: false, fleet: true },
    Path { origin: "tyn", dest: "tun", army: false, fleet: true },
    Path { origin: "tyn", dest: "wes", army: false, fleet: true },
    Path { origin: "tyn", dest: "gol", army: false, fleet: true },
    Path { origin: "tyn", dest: "nap", army: false, fleet: true },
    Path { origin: "ukr", dest: "sev", army: true, fleet: false },
    Path { origin: "ukr", dest: "mos", army: true, fleet: false },
    Path { origin: "ukr", dest: "rum", army: true, fleet: false },
    Path { origin: "ukr", dest: "gal", army: true, fleet: false },
    Path { origin: "ukr", dest: "war", army: true, fleet: false },
    Path { origin: "ven", dest: "tri", army: true, fleet: true },
    Path { origin: "ven", dest: "adr", army: false, fleet: true },
    Path { origin: "ven", dest: "apu", army: true, fleet: true },
    Path { origin: "ven", dest: "tyr", army: true, fleet: false },
    Path { origin: "ven", dest: "pie", army: true, fleet: false },
    Path { origin: "ven", dest: "rom", army: true, fleet: false },
    Path { origin: "ven", dest: "tus", army: true, fleet: false },
    Path { origin: "vie", dest: "gal", army: true, fleet: false },
    Path { origin: "vie", dest: "tyr", army: true, fleet: false },
    Path { origin: "vie", dest: "tri", army: true, fleet: false },
    Path { origin: "vie", dest: "boh", army: true, fleet: false },
    Path { origin: "vie", dest: "bud", army: true, fleet: false },
    Path { origin: "wal", dest: "eng", army: false, fleet: true },
    Path { origin: "wal", dest: "lvp", army: true, fleet: true },
    Path { origin: "wal", dest: "iri", army: false, fleet: true },
    Path { origin: "wal", dest: "lon", army: true, fleet: true },
    Path { origin: "wal", dest: "yor", army: true, fleet: false },
    Path { origin: "war", dest: "ukr", army: true, fleet: false },
    Path { origin: "war", dest: "pru", army: true, fleet: false },
    Path { origin: "war", dest: "gal", army: true, fleet: false },
    Path { origin: "war", dest: "mos", army: true, fleet: false },
    Path { origin: "war", dest: "sil", army: true, fleet: false },
    Path { origin: "war", dest: "lvn", army: true, fleet: false },
    Path { origin: "wes", dest: "tyn", army: false, fleet: true },
    Path { origin: "wes", dest: "mid", army: false, fleet: true },
    Path { origin: "wes", dest: "tun", army: false, fleet: true },
    Path { origin: "wes", dest: "spa_sc", army: false, fleet: true },
    Path { origin: "wes", dest: "naf", army: false, fleet: true },
    Path { origin: "wes", dest: "gol", army: false, fleet: true },
    Path { origin: "yor", dest: "edi", army: true, fleet: true },
    Path { origin: "yor", dest: "wal", army: true, fleet: false },
    Path { origin: "yor", dest: "lon", army: true, fleet: true },
    Path { origin: "yor", dest: "nth", army: false, fleet: true },
    Path { origin: "yor", dest: "lvp", army: true, fleet: false },
];

/// 経路情報の定数配列の実装
impl Path {
    ///
    /// 2つの地名コードが隣接しているか判定する（origin→dest方向のみ）
    ///
    pub(crate) fn is_adjacent(origin: &str, dest: &str) -> bool {
        PATHS.iter().any(|p| p.origin == origin && p.dest[..3] == dest[..3])
    }

    ///
    /// ユニットが指定地点に存在可能かを判定する
    ///
    pub(crate) fn can_unit_exist_at(unit: &Unit, code: &str) -> bool {
        match unit.kind() {
            UnitKind::Army(_) => PATHS.iter().any(|p| p.origin == code && p.army),
            UnitKind::Fleet(_) => PATHS.iter().any(|p| p.origin == code && p.fleet),
        }
    }

    ///
    /// ユニットが指定地点に移動可能かを判定する
    ///
    pub(crate) fn can_unit_move_to(unit: &Unit, dest: &str, via_convoy: bool) -> bool {
        match unit.kind() {
            UnitKind::Army(_) => PATHS
                .iter()
                .any(|p| p.origin == unit.location.code_with_coast() && p.dest == dest && p.army && !via_convoy),
            UnitKind::Fleet(_) => PATHS
                .iter()
                .any(|p| p.origin == unit.location.code_with_coast() && p.dest == dest && p.fleet),
        }
    }

    ///
    /// ユニットが指定地点へのサポートが可能かを判定する
    ///
    pub(crate) fn can_unit_support_to(unit: &Unit, origin: &str, dest: &str) -> bool {
        match unit.kind() {
            UnitKind::Army(_) => PATHS.iter().any(|p| p.origin == origin && p.dest == dest && p.army),
            UnitKind::Fleet(_) => PATHS
                .iter()
                .any(|p| p.origin == origin && p.dest[..3] == dest[..3] && p.fleet),
        }
    }

    ///
    /// origin から dest への輸送移動が可能かを判定する
    ///
    pub(crate) fn can_convoy_move(origin: &str, dest: &str) -> bool {
        PATHS
            .iter()
            .any(|p| p.origin[..3] == origin[..3] && p.dest[..3] == dest[..3] && p.fleet)
    }

    ///
    /// 許可水域だけを通って origin から dest まで到達可能か判定する
    ///
    pub(crate) fn is_reachable_by_sea(origin: &str, dest: &str, allowed_waters: &HashSet<&str>) -> bool {
        let mut visited: HashSet<&str> = HashSet::new();
        let mut queue: VecDeque<&str> = VecDeque::new();

        // origion が水域の場合は dest が隣接していても true を返す
        if Province::from_code(origin).expect("valid province code").is_water() && Self::can_convoy_move(origin, dest) {
            return true;
        };

        // 初期起点を集める
        for p in PATHS.iter().filter(|p| p.fleet && &p.origin[..3] == origin) {
            if allowed_waters.contains(p.dest) {
                queue.push_back(p.dest);
            }
        }

        // BFS でチェーン探索
        while let Some(current) = queue.pop_front() {
            if !visited.insert(current) {
                continue;
            }
            for p in PATHS.iter().filter(|p| p.fleet && p.origin == current) {
                if &p.dest[..3] == dest {
                    return true;
                }
                if allowed_waters.contains(p.dest) && !visited.contains(p.dest) {
                    queue.push_back(p.dest);
                }
            }
        }

        false
    }
}

// ============================================================================
// tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn count_paths() -> usize {
        PATHS.len()
    }

    #[test]
    fn test_path_count() {
        assert_eq!(count_paths(), 436);
    }
}
