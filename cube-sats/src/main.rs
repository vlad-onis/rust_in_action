#[derive(Debug)]
pub struct CubeSat {
    id: u64,
    mailbox:Mailbox,
}

#[derive(Debug)]
pub struct Mailbox {
    messages: Vec<Message>,
}

#[derive(Debug)]
enum StatusMessage {
    Ok,
}

type Message = String;

struct GroundStation {
    age: u64,
}

impl GroundStation {
    pub fn send(&self, to: &mut CubeSat, message: Message) {
        to.mailbox.messages.push(message);
    }
}

impl CubeSat {
    pub fn recv(&mut self) -> Option<Message>{
        self.mailbox.messages.pop()
    }
}

impl Copy for GroundStation {}

impl Clone for GroundStation {
    fn clone(&self) -> Self {
        println!("Cloning Groundstation: {}", self.age);
        GroundStation {
            age: self.age
        }
    }
}

fn copy_test(gs: GroundStation) {
    println!("Copied: {}", gs.age);
}

pub fn simple_message() {
    let base = GroundStation {
        age: 3
    };

    copy_test(base);

    let mut sat_a = CubeSat {
        id: 1,
        mailbox: Mailbox { messages: vec![] },
    };

    println!("Sat_a: {:?}", sat_a);

    base.send(&mut sat_a, Message::from("Hello from base"));

    let from_sat_a = sat_a.recv();
    println!("From satelite a: {}", from_sat_a.unwrap_or(Message::from("No messages yet")));

    let from_sat_a = sat_a.recv();
    println!("From satelite a: {}", from_sat_a.unwrap_or(Message::from("No other messages yet")));
}

fn main() {
    simple_message();
}
