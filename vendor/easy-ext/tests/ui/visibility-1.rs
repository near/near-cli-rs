use easy_ext::ext;

#[ext(AssocLevel1)]
impl str {
    pub const ASSOC1: u8 = 1;
    const ASSOC2: u8 = 2; //~ ERROR all associated items must have a visibility of `pub`

    pub fn assoc(&self) {}
}

#[ext(AssocLevel2)]
impl str {
    fn assoc1(&self) {}

    pub fn assoc2(&self) {} //~ ERROR all associated items must have inherited visibility
}

#[ext(pub ImplLevel1)]
impl str {
    fn assoc1(&self) {}

    pub fn assoc2(&self) {} //~ ERROR all associated items must have inherited visibility
}

fn main() {}
