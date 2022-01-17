
use jset_desk::cx::Cx;
use jset_desk::image::IterType;

fn main() {
    let a = Cx::rect(1.0, 0.0);
    let b = Cx::rect(0.707, 0.707);
    let c = Cx::rect(0.0, 1.0);
    
    let it0 = IterType::Mandlebrot;
    let it1 = IterType::Mandlebrot;
    let it2 = IterType::PseudoMandlebrot(a, b);
    let it3 = IterType::PseudoMandlebrot(a, b);
    let it4 = IterType::PseudoMandlebrot(b, c);
    let it5 = IterType::Polynomial(vec![a, b]);
    let it6 = IterType::Polynomial(vec![a, b]);
    let it7 = IterType::Polynomial(vec![a, c]);
    
    println!("{:?} == {:?}, {} {}", &it0, &it1, it0 == it1, &it0 == &it1);
    println!("{:?} == {:?}, {} {}", &it0, &it2, it0 == it2, &it0 == &it2);
    println!("{:?} == {:?}, {} {}", &it2, &it3, it2 == it3, &it2 == &it3);
    println!("{:?} == {:?}, {} {}", &it2, &it4, it2 == it4, &it2 == &it4);
    println!("{:?} == {:?}, {} {}", &it5, &it6, it5 == it6, &it5 == &it6);
    println!("{:?} == {:?}, {} {}", &it5, &it7, it5 == it7, &it5 == &it7);
}