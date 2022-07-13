use rand::{prelude::SliceRandom, Rng};
use std::fmt::Debug;

#[derive(Debug)]
struct Dwarf;

#[derive(Debug)]
struct Human;

#[derive(Debug)]
struct Elf;

#[derive(Debug)]
enum Thing {
    Sword,
    Trinket,
}

trait Enchanter: Debug {
    fn competency(&self) -> f64;

    fn enchant(&self, thing: &mut Thing) {
        let prob_of_success = self.competency();
        let spell_is_successful = rand::thread_rng().gen_bool(prob_of_success);

        println!("{:?} mutters incoherently", self);

        if spell_is_successful {
            println!("The {:?} glows brightly. ", thing);
        } else {
            println!(
                "The {:?} fizzes, then turns into a worthless trinket",
                thing
            );
            *thing = Thing::Trinket;
        }
    }
}

impl Enchanter for Dwarf {
    fn competency(&self) -> f64 {
        0.50
    }
}

impl Enchanter for Human {
    fn competency(&self) -> f64 {
        0.75
    }
}

impl Enchanter for Elf {
    fn competency(&self) -> f64 {
        0.90
    }
}

fn main() {
    let d = Dwarf;
    let e = Elf;
    let h = Human;

    let party: Vec<&dyn Enchanter> = vec![&d, &e, &h];
    let spellcaster = party.choose(&mut rand::thread_rng()).unwrap();

    spellcaster.enchant(&mut Thing::Sword);
    println!("The spellcaster: {:?}", spellcaster);
}
