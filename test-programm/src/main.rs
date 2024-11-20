fn main() {
    let mut count = 0;

    for i in 0.. 10 {
        count += i;
    }

    println!("{}", square(square(count)));

}

fn square(n: i32) -> i32 {
    return n * n;
}