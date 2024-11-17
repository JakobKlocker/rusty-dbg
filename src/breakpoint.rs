pub struct Breakpoint{
    breakpoint: Vec<u64>,
}

impl Breakpoint{
    pub fn new() -> Self{
        Breakpoint{
            breakpoint: Vec::new(),
        }
    }

    pub fn add_breakpoint(mut self, addr: u64){
        self.breakpoint.push(addr);
        println!("added bp {}", addr);
    }

    pub fn remove_breakpoint(mut self, addr: u64) -> bool {
        if let Some(index) = self.breakpoint.iter().position(|&x| x == addr){
            self.breakpoint.remove(index);
            println!("found bp, removed {}", addr);
            return true;
        }
        false
    }
}