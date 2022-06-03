pub static S1: u8 = 0b1111000;
pub static S2: u8 = 0b0101110;
pub static S3: u8 = 0b0011011;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Operator {
    Intersection,
    Union,
    Difference,
}

impl Operator {
    fn apply(&self, lhs: u8, rhs: u8) -> u8 {
        match self {
            Operator::Intersection => lhs & rhs,
            Operator::Union => lhs | rhs,
            Operator::Difference => (lhs ^ rhs) & lhs,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Term {
    Unary(u8),
    Binary(Box<Term>, Operator, Box<Term>),
}

impl Term {
    fn eval(&self) -> u8 {
        match self {
            Term::Unary(s) => *s,
            Term::Binary(lhs, op, rhs) => op.apply(lhs.eval(), rhs.eval()),
        }
    }

    pub fn info(&self) -> RenderInfo {
        match self {
            Term::Unary(regions) => {
                return RenderInfo {
                    width: BOX_WIDTH,
                    height: BOX_HEIGHT,
                    count: 1,
                    regions: *regions,
                };
            }
            Term::Binary(l, op, r) => {
                let li = l.info();
                let ri = r.info();
                return RenderInfo {
                    width: li.width + OP_WIDTH + ri.width,
                    height: f64::max(li.height, ri.height) + OP_HEIGHT + BOX_HEIGHT,
                    count: li.count + 1 + ri.count,
                    regions: op.apply(li.regions, ri.regions),
                }
            }
        }
    }

    fn render(&self, id: usize, x: f64, y: f64, down: bool, root: bool) -> (String, ViewBox) {
        match self {
            Term::Unary(regions) => {
                return (
                    venn3(id, x, y, *regions),
                    ViewBox {
                        x,
                        y,
                        width: BOX_WIDTH,
                        height: BOX_HEIGHT,
                    },
                );
            }
            Term::Binary(l, op, r) => {
                let selfi = self.info();
                let li = l.info();
                let ri = r.info();
                if down {
                    let x_left = x + (BOX_WIDTH / 2.0) - ((OP_WIDTH / 2.0) + li.width);
                    let x_right = x + (BOX_WIDTH / 2.0) + ((OP_WIDTH / 2.0));
                    let y_left = y + BOX_HEIGHT + OP_HEIGHT;
                    let y_right = y + BOX_HEIGHT + OP_HEIGHT;
                    let ls = l.render(
                        id,
                        x_left,
                        y_left,
                        down,
                        false,
                    );
                    let self_s = if root {"".to_string()} else {Term::Unary(selfi.regions).render(id + li.count, x, y, down, root).0};
                    let rs = r.render(
                        id + li.count + if root {0} else {1},
                        x_right,
                        y_right,
                        down,
                        false,
                    );
                    return (
                        format!("{}{}{}", ls.0, rs.0, self_s),
                        ViewBox {
                            x: x_left,
                            y: f64::min(y_left, y_right) - (BOX_HEIGHT + OP_HEIGHT),
                            width: li.width + BOX_WIDTH + ri.width,
                            height: f64::max(li.height, ri.height) + OP_HEIGHT + BOX_HEIGHT,
                        },
                    );
                } else {
                    let x_left = x + (BOX_WIDTH / 2.0) - ((OP_WIDTH / 2.0) + li.width);
                    let x_right = x + (BOX_WIDTH / 2.0) + ((OP_WIDTH / 2.0));
                    // let y_left = y - (OP_HEIGHT + li.height);
                    // let y_right = y - (OP_HEIGHT + ri.height);
                    let y_left = y - (OP_HEIGHT + BOX_HEIGHT);
                    let y_right = y - (OP_HEIGHT + BOX_HEIGHT);
                    let ls = l.render(
                        id,
                        x_left,
                        y_left,
                        down,
                        false,
                    );
                    let self_s = Term::Unary(selfi.regions).render(id + li.count, x, y, down, root).0;
                    let rs = r.render(
                        id + li.count + 1,
                        x_right,
                        y_right,
                        down,
                        false,
                    );
                    let height = f64::max(li.height, ri.height) + OP_HEIGHT + BOX_HEIGHT;
                    return (
                        format!("{}{}{}", ls.0, rs.0, self_s),
                        ViewBox {
                            x: x_left,
                            y: y - height,
                            width: li.width + BOX_WIDTH + ri.width,
                            height,
                        },
                    );
                }
            }
        }
    }
}

pub struct RenderInfo {
    width: f64,
    height: f64,
    pub count: usize,
    regions: u8,
}

struct ViewBox {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

pub fn render_equation(id: usize, lhs: &Term, rhs: &Term) -> String {
    let (ls, li) = lhs.render(id, 0.0, 0.0, false, true);
    let (rs, ri) = rhs.render(id + lhs.info().count, 0.0, 0.0, true, true);
    return format!(
        r###"<svg class="venn vennop" version="1.1" viewBox="{} {} {} {}" preserveAspectRatio="xMidYMid meet" xmlns="http://www.w3.org/2000/svg">
    {}
    {}
</svg>"###,
        li.x, li.y + (BOX_HEIGHT / 2.0), f64::max(li.width, ri.width), (li.height + ri.height) - BOX_HEIGHT,
        ls,
        rs,
    );
}

static OP_WIDTH: f64 = 20.0;
static OP_HEIGHT: f64 = 10.0;

static BOX_X: f64 = -55.0;
static BOX_Y: f64 = -60.0;
static BOX_WIDTH: f64 = 110.0;
static BOX_HEIGHT: f64 = 110.0;

fn venn3(id: usize, x: f64, y: f64, regions: u8) -> String {
    let r = 30;
    let a_cx = -17.3205 + x;
    let a_cy = 10.0 + y;
    let b_cx = 0.0 + x;
    let b_cy = -20.0 + y;
    let c_cx = 17.3205 + x;
    let c_cy = 10.0 + y;
    let a_tx = -25.9807 + x;
    let a_ty = 15.0 + y;
    let b_tx = -0.0 + x;
    let b_ty = -30.0 + y;
    let c_tx = 25.9807 + x;
    let c_ty = 15.0 + y;

    return format!(
        r###"
          <!-- A -->
          <clipPath id="{}_clip_a">
            <circle cx="{}" cy="{}" r="{}"/>
          </clipPath>

          <!-- B -->
          <clipPath id="{}_clip_b">
            <circle cx="{}" cy="{}" r="{}"/>
          </clipPath>

          <!-- C -->
          <clipPath id="{}_clip_c">
            <circle cx="{}" cy="{}" r="{}"/>
          </clipPath>

          <!-- A intersect B -->
          <clipPath id="{}_clip_aib" clip-path="url('#{}_clip_a')">
            <circle cx="{}" cy="{}" r="{}"/>
          </clipPath>

          <!-- A intersect C -->
          <clipPath id="{}_clip_aic" clip-path="url('#{}_clip_a')">
            <circle cx="{}" cy="{}" r="{}"/>
          </clipPath>

          <!-- B intersect C -->
          <clipPath id="{}_clip_bic" clip-path="url('#{}_clip_c')">
            <circle cx="{}" cy="{}" r="{}"/>
          </clipPath>

          <!-- A intersect B intersect C -->
          <clipPath id="{}_clip_aibic" clip-path="url('#{}_clip_aib')">
            <circle cx="{}" cy="{}" r="{}"/>
          </clipPath>

          <!-- a intersect B intersect C -->
          <mask id="{}_mask_aibic" >
            <rect x="{}" y="{}" width="{}" height="{}" style="fill: white;"/>
            <rect clip-path="url('#{}_clip_aibic')" x="{}" y="{}" width="{}" height="{}" style="fill: black;"/>
          </mask>

          <!-- A union B -->
          <mask id="{}_mask_aub">
            <rect x="{}" y="{}" width="{}" height="{}" style="fill: white;"/>
            <circle cx="{}" cy="{}" r="{}" style="fill: black;"/>
            <circle cx="{}" cy="{}" r="{}" style="fill: black;"/>
          </mask>

          <!-- A union C -->
          <mask id="{}_mask_auc">
            <rect x="{}" y="{}" width="{}" height="{}" style="fill: white;"/>
            <circle cx="{}" cy="{}" r="{}" style="fill: black;"/>
            <circle cx="{}" cy="{}" r="{}" style="fill: black;"/>
          </mask>

          <!-- B union C -->
          <mask id="{}_mask_buc">
            <rect x="{}" y="{}" width="{}" height="{}" style="fill: white;"/>
            <circle cx="{}" cy="{}" r="{}" style="fill: black;"/>
            <circle cx="{}" cy="{}" r="{}" style="fill: black;"/>
          </mask>

          <!-- just A -->
          <circle clip-path="url('#{}_clip_a')" mask="url(#{}_mask_buc)" cx="{}" cy="{}" r="{}"{}/>
          <!-- just B -->
          <circle clip-path="url('#{}_clip_b')" mask="url(#{}_mask_auc)" cx="{}" cy="{}" r="{}"{}/>
          <!-- just C -->
          <circle clip-path="url('#{}_clip_c')" mask="url(#{}_mask_aub)" cx="{}" cy="{}" r="{}"{}/>
          <!-- A intersect B without C -->
          <circle clip-path="url('#{}_clip_aib')" mask="url(#{}_mask_aibic)" cx="{}" cy="{}" r="{}"{}/>
          <!-- A intersect C without B -->
          <circle clip-path="url('#{}_clip_aic')" mask="url(#{}_mask_aibic)" cx="{}" cy="{}" r="{}"{}/>
          <!-- B intersect C without A -->
          <circle clip-path="url('#{}_clip_bic')" mask="url(#{}_mask_aibic)" cx="{}" cy="{}" r="{}"{}/>
          <!-- A intersect B intersect C -->
          <circle clip-path="url('#{}_clip_aibic')" cx="{}" cy="{}" r="{}"{}/>

          <circle cx="{}" cy="{}" r="{}"/>
          <text x="{}" y="{}">A</text>
          <circle cx="{}" cy="{}" r="{}"/>
          <text x="{}" y="{}">B</text>
          <circle cx="{}" cy="{}" r="{}"/>
          <text x="{}" y="{}">C</text>"###,
        id, a_cx, a_cy, r,
        id, b_cx, b_cy, r,
        id, c_cx, c_cy, r,
        id, id, b_cx, b_cy, r,
        id, id, c_cx, c_cy, r,
        id, id, b_cx, b_cy, r,
        id, id, c_cx, c_cy, r,
        id, BOX_X, BOX_Y, BOX_WIDTH, BOX_HEIGHT, id, BOX_X, BOX_Y, BOX_WIDTH, BOX_HEIGHT,
        id, BOX_X, BOX_Y, BOX_WIDTH, BOX_HEIGHT, a_cx, a_cy, r, b_cx, b_cy, r,
        id, BOX_X, BOX_Y, BOX_WIDTH, BOX_HEIGHT, a_cx, a_cy, r, c_cx, c_cy, r,
        id, BOX_X, BOX_Y, BOX_WIDTH, BOX_HEIGHT, b_cx, b_cy, r, c_cx, c_cy, r,
        id, id, x, y, (BOX_WIDTH / 2.0) - 2.0, if (regions & 0b0000001) != 0 {r###"class="venn_yay""###} else {""},
        id, id, x, y, (BOX_WIDTH / 2.0) - 2.0, if (regions & 0b0010000) != 0 {r###"class="venn_yay""###} else {""},
        id, id, x, y, (BOX_WIDTH / 2.0) - 2.0, if (regions & 0b1000000) != 0 {r###"class="venn_yay""###} else {""},
        id, id, x, y, (BOX_WIDTH / 2.0) - 2.0, if (regions & 0b0000010) != 0 {r###"class="venn_yay""###} else {""},
        id, id, x, y, (BOX_WIDTH / 2.0) - 2.0, if (regions & 0b0000100) != 0 {r###"class="venn_yay""###} else {""},
        id, id, x, y, (BOX_WIDTH / 2.0) - 2.0, if (regions & 0b0100000) != 0 {r###"class="venn_yay""###} else {""},
        id, x, y, (BOX_WIDTH / 2.0) - 2.0, if (regions & 0b0001000) != 0 {r###"class="venn_yay""###} else {""},
        a_cx, a_cy, r, a_tx, a_ty,
        b_cx, b_cy, r, b_tx, b_ty,
        c_cx, c_cy, r, c_tx, c_ty,
    );
}
