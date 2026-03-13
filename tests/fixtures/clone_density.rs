// Test fixture for clone-density rule

fn too_many_clones() {
    let a = String::new();
    let _b = a.clone();
    let _c = a.clone();
    let _d = a.clone();
    let _e = a.clone();
    let _f = a.clone();
    let _g = a.clone();
}

fn clean_function() {
    let a = String::from("hello");
    let _b = a.clone();
    let _c = 1;
    let _d = 2;
    let _e = 3;
    let _f = 4;
    let _g = 5;
    let _h = 6;
    let _i = 7;
    let _j = 8;
    let _k = 9;
    let _l = 10;
    let _m = 11;
    let _n = 12;
    let _o = 13;
    let _p = 14;
    let _q = 15;
    let _r = 16;
}

struct MyStruct {
    data: String,
}

impl MyStruct {
    fn clone_heavy_method(&self) {
        let _a = self.data.clone();
        let _b = self.data.clone();
        let _c = self.data.clone();
        let _d = self.data.clone();
        let _e = self.data.clone();
        let _f = self.data.clone();
    }
}

fn empty_function() {}
