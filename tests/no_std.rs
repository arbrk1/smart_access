#![no_std]

use smart_access::Cps;

#[test]
fn test1() {
    let mut foo = 3;

    foo.batch_ct().add(|x,_| { *x += 1; } ).run();

    assert!(foo == 4);
}

#[test]
fn test2() {
    let mut foo = [1, 2, 3, 4, 5, 6];

    let bar = (&mut foo[..]).at(0).replace(7);
    assert!(foo == [7, 2, 3, 4, 5, 6]);
    assert!(bar == Some(1));
}


// copied almost verbatim from core_impls.rs

#[test]
fn test_slice() {
    let mut foo = [1,2,3,4,5];

    let update = |i| move |slice: &mut [i32]| {
        let old_value = slice[0];
        
        slice[0] = i;

        old_value
    };
    
    assert!((&mut foo[..]).at(1..3).access(update(6)) == Some(2));
    assert!(foo == [1,6,3,4,5]);
    
    assert!((&mut foo[..]).at(2..).access(update(7)) == Some(3));
    assert!(foo == [1,6,7,4,5]);
    
    assert!((&mut foo[..]).at(..4).access(update(8)) == Some(1));
    assert!(foo == [8,6,7,4,5]);
    
    assert!((&mut foo[..]).at(..).access(update(9)) == Some(8));
    assert!(foo == [9,6,7,4,5]);
    
    assert!((&mut foo[..]).at(..=6).access(update(1)) == None);
    assert!(foo == [9,6,7,4,5]);
    
    assert!((&mut foo[..]).at(3..=4).access(update(0)) == Some(4));
    assert!(foo == [9,6,7,0,5]);

    assert!((&mut foo[..]).at(4).replace(1) == Some(5));
    assert!(foo == [9,6,7,0,1]);
}

#[test]
fn test_optional() {
    let mut foo: Option<i32> = Some(0);
    let mut bar: Option<i32> = None;

    assert!(foo.at(()).replace(1) == Some(0));
    assert!(foo == Some(1));
    assert!(bar.at(()).replace(2) == None);
    assert!(bar == None);

    let mut foo: Result<i32,i32> = Ok(0);
    let mut bar: Result<i32,i32> = Err(1);
    
    assert!(foo.at(()).replace(1) == Some(0));
    assert!(foo == Ok(1));
    assert!(bar.at(()).replace(2) == None);
    assert!(bar == Err(1));
}
