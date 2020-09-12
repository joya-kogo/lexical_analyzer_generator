mod analyzer{

    struct Analyzer {
        state: u64
    }

    impl Analyzer {
        fn read_token(&mut self, ch:char) {
if self.state == 1{
match ch {
'l'=>self.state = 2,
_ => (),
}
}
if self.state == 2{
match ch {
'l'=>self.state = 3,
'n'=>self.state = 4,
_ => (),
}
}
if self.state == 3{
match ch {
'l'=>self.state = 3,
'n'=>self.state = 4,
_ => (),
}
}
        }
    }

}

fn main() {

}
