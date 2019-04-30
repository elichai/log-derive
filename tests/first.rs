use log_derive::logfn;
#[logfn(INFO, fmt = "DB: {:?}", ok="debug", err="trace")]
fn wrapped_function(a: u8, b: &str) {
    let mut test1 = Vec::new();
    let mut test2 = || {
        test1.push(5);
    };
    test2();
    println!("{} {}", b, a);
}

struct AAAAAA;
impl AAAAAA {
    #[logfn(Info)]
    pub fn yoyoy(&self, a: String, b: u8, c: Vec<u8>) -> Vec<u8> {
        vec![0u8; 8]
    }
}

#[derive(Debug)]
struct E;

trait Test {
    fn abc(&mut self, err: Tes) -> Result<String, E>;
    fn third(&self, err: &Tes) -> Result<String, E>;
}

struct Me(Option<u8>);
struct Tes(pub bool);

impl Test for Me {

//    #[logfn(Info)]
    #[logfn(INFO, fmt = "DB: {:?}", ok="debug", err="trace")]
    fn abc(&mut self, err: Tes) -> Result<String, E> {
        let mut clos = || {
            self.third(&err)?;
            if err.0 {
                return Err(E)
            } else {
                self.0 = Some(5);
                return Ok(String::from("Hi!"))
            }
        };
        let result = clos();
        result

    }

    #[logfn(Info)]
    fn third(&self, err: &Tes) -> Result<String, E> {
        if err.0 {
            return Err(E)
        } else {
            return Ok(String::from("Hi!"))
        }
    }

    #[logfn_inputs]
    fn just_inputs(&self, err: &Tes) -> Result<String, E> {
        if err.0 {
            return Err(E);
        } else {
            return Ok(String::from("Hi!"));
        }
    }

    #[logfn_inputs]
    #[logfn(Info)]
    fn both(&self, err: &Tes) -> Result<String, E> {
        let mut clos = || {
            self.third(&err)?;
            if err.0 {
                return Err(E);
            } else {
                return Ok(String::from("Hi!"));
            }
        };
        let result = clos();
        result
    }
}


#[test]
fn works() {
    wrapped_function(5, "cool!");
    let a = AAAAAA;
    let _ = a.yoyoy(String::from("fds"), 55, vec![1u8; 12]);
    let mut b = Me(None);
    let tes = Tes(false);
    b.abc(tes).unwrap();
}

#[test]
fn test_inputs() {
    let b = Me(Some(5));
    let tes = Tes(false);
    b.just_inputs(&tes).unwrap();
    b.both(&tes).unwrap();

}

#[test]
#[should_panic]
fn fail() {
    wrapped_function(5, "cool!");
    let a = AAAAAA;
    let _ = a.yoyoy(String::from("fds"), 55, vec![1u8; 12]);
    let mut b = Me(None);
    let tes = Tes(true);
    b.abc(tes).unwrap();
}