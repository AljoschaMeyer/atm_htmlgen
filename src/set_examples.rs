use std::cmp::max;

pub static S1: u8 = 0b1111000;
pub static S2: u8 = 0b0101110;
pub static S3: u8 = 0b0011011;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Operator {
    Intersection,
    Union,
    Difference,
    SymmetricDifference,
}

impl Operator {
    fn apply(&self, lhs: u8, rhs: u8) -> u8 {
        match self {
            Operator::Intersection => lhs & rhs,
            Operator::Union => lhs | rhs,
            Operator::Difference => (lhs ^ rhs) & lhs,
            Operator::SymmetricDifference => lhs ^ rhs,
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

    pub fn count(&self) -> usize {
        match self {
            Term::Unary(_) => 1,
            Term::Binary(lhs, _, rhs) => lhs.count() + 1 + rhs.count(),
        }
    }

    pub fn to_coordinate_tree(&self, minsep: f64) -> Node {
        let mut n = self.to_coordinate_tree_();
        setup(&mut n, minsep);
        petrify(&mut n);
        return n;
    }

    fn to_coordinate_tree_(&self) -> Node {
        match self {
            Term::Unary(_) => return Node::new_leaf(),
            Term::Binary(l, _, r) => {
                return Node::new_lr(
                    Box::into_raw(Box::new(l.to_coordinate_tree_())),
                    Box::into_raw(Box::new(r.to_coordinate_tree_())),
                )
            }
        }
    }

    fn render(&self, id: usize, coordinates: &Node, x: f64, y: f64, skip: bool, down: bool, draw: DrawingInfo) -> Out {
        match self {
            Term::Unary(regions) => {
                if skip {
                    return Out::Text("".into());
                } else {
                    return Out::Text((draw.draw_node)(id, x, y, *regions).into());
                }
            }
            Term::Binary(l_t, op, r_t) => {
                let down_signum = if down {1.0} else {-1.0};
                let y_offset = (draw.node_height + draw.op_height) * down_signum;
                let next_y = y + y_offset;
                let l_dimensions = unsafe {(*coordinates.llink).dimensions()};
                let l_x = unsafe{(*coordinates.llink).xcoord};
                let r_x = unsafe{(*coordinates.rlink).xcoord};
                let regions = self.eval();
                let tip_y = y + (down_signum * ((draw.node_height / 2.0) + FLOW_MARGIN));
                let arrow_base_y = tip_y + (down_signum * ARROW_HEIGHT);

                let (op_id, op_tex) = match op {
                    Operator::Intersection => ("intersection", r###"\cap"###),
                    Operator::Union => ("union", r###"\cup"###),
                    Operator::Difference => ("setminus", r###"\setminus"###),
                    Operator::SymmetricDifference => ("symdif", r###"\operatorname{\triangle}"###),
                };

                return Out::Many(vec![
                    l_t.render(id, unsafe{&(*coordinates.llink)}, l_x, next_y, false, down, draw),
                    if skip {
                        Out::Text("".into())
                    } else {
                        Out::Text((draw.draw_node)(id + l_dimensions.count, x, y, regions).into())
                    },
                    r_t.render(id + l_dimensions.count + 1, unsafe{&(*coordinates.rlink)}, r_x, next_y, false, down, draw),
                    Out::Text(format!(
                        r###"<line x1="{}" y1="{}" x2="{}" y2="{}" class="venn_flow"/>
                        <line x1="{}" y1="{}" x2="{}" y2="{}" class="venn_flow"/>
                        <polygon points="{},{} {},{} {},{}" class="venn_flow"/>
                        <foreignObject x="{}" y="{}" width="100" height="100"><div class="venn_op_container">"###,
                        l_x + (draw.node_width / 2.0) + FLOW_MARGIN,
                        next_y,
                        r_x - ((draw.node_width / 2.0) + FLOW_MARGIN),
                        next_y,

                        x, next_y, x, tip_y,

                        x, tip_y, x - (ARROW_WIDTH / 2.0), arrow_base_y, x + (ARROW_WIDTH / 2.0), arrow_base_y,

                        x - 50.0, next_y - 50.0,
                    ).into()),
                    Out::TeX(crate::macros::TeX::default(), vec![Out::MathMacro(crate::macros::MathMacro::default(), vec![], op_id.into(), op_tex.into())], false),
                    Out::Text("</div></foreignObject>".into()),
                ]);
            }
        }
    }
}

pub struct Dimensions {
    max_y: usize,
    min_x: f64,
    max_x: f64,
    count: usize,
}

#[derive(Clone, Copy)]
pub struct DrawingInfo {
    node_width: f64,
    node_height: f64,
    op_width: f64,
    op_height: f64,
    draw_node: fn(usize, f64, f64, u8) -> String,
}

pub fn render_venn(id: usize, t: &Term, draw: DrawingInfo) -> Out {
    let coordinates = Box::new(t.to_coordinate_tree(draw.node_width + draw.op_width));
    let dimensions = coordinates.dimensions();

    let x_start = dimensions.min_x - (draw.node_width / 2.0);
    let x_end = dimensions.max_x + (draw.node_width / 2.0);
    let width = (-1.0 * x_start) + x_end;

    let y_start = -1.0 * ((dimensions.max_y as f64 * (draw.node_height + draw.op_height)) + (draw.node_height / 2.0));
    let y_end = draw.node_height / 2.0;
    let height = (-1.0 * y_start) + y_end;

    let r = Out::Many(vec![
        Out::Text(format!(
            r###"<svg class="venn vennop multi" version"1.1" viewBox="{} {} {} {}" xmlns="http://www.w3.org/2000/svg">"###,
            x_start - 1.0, y_start - 1.0, width + 2.0, height + 2.0,
        ).into()),
        t.render(id, &coordinates, 0.0, 0.0, false, false, draw),
        Out::Text("</svg>".into()),
    ]);

    Node::free(Box::into_raw(coordinates));

    return r;
}

pub fn render_equation(id: usize, lhs: &Term, rhs: &Term, draw: DrawingInfo) -> Out {
    let coordinates_lhs = Box::new(lhs.to_coordinate_tree(draw.node_width + draw.op_width));
    let dimensions_lhs = coordinates_lhs.dimensions();
    let coordinates_rhs = Box::new(rhs.to_coordinate_tree(draw.node_width + draw.op_width));
    let dimensions_rhs = coordinates_rhs.dimensions();

    let x_start = f64::min(dimensions_lhs.min_x, dimensions_rhs.min_x) - (draw.node_width / 2.0);
    let x_end = f64::max(dimensions_lhs.max_x, dimensions_rhs.max_x) + (draw.node_width / 2.0);
    let width = (-1.0 * x_start) + x_end;

    let y_start = -1.0 * ((dimensions_lhs.max_y as f64 * (draw.node_height + draw.op_height)) + (draw.node_height / 2.0));
    let y_end = (dimensions_rhs.max_y as f64 * (draw.node_height + draw.op_height)) + (draw.node_height / 2.0);
    let height = (-1.0 * y_start) + y_end;

    let r = Out::Many(vec![
        Out::Text(format!(
            r###"<svg class="venn vennop multi" version"1.1" viewBox="{} {} {} {}" xmlns="http://www.w3.org/2000/svg">"###,
            x_start - 1.0, y_start - 1.0, width + 2.0, height + 2.0,
        ).into()),
        lhs.render(id, &coordinates_lhs, 0.0, 0.0, false, false, draw),
        rhs.render(id + dimensions_lhs.count, &coordinates_rhs, 0.0, 0.0, true, true, draw),
        Out::Text("</svg>".into()),
    ]);

    Node::free(Box::into_raw(coordinates_lhs));
    Node::free(Box::into_raw(coordinates_rhs));

    return r;
}

pub static DRAW_VENN3: DrawingInfo = DrawingInfo {
    node_width: BOX_WIDTH,
    node_height: BOX_HEIGHT,
    op_width: OP_WIDTH,
    op_height: OP_HEIGHT,
    draw_node: venn3,
};

pub static DRAW_VENN2: DrawingInfo = DrawingInfo {
    node_width: BOX_WIDTH,
    node_height: BOX_HEIGHT2,
    op_width: OP_WIDTH,
    op_height: OP_HEIGHT,
    draw_node: venn2,
};

static OP_WIDTH: f64 = 60.0;
static OP_HEIGHT: f64 = 20.0;

static BOX_WIDTH: f64 = 110.0;
static BOX_HEIGHT: f64 = 110.0;
static FRAME_OFFSET: f64 = 5.0;
static BOX_HEIGHT2: f64 = 80.0;

static FLOW_MARGIN: f64 = 5.0;
static ARROW_WIDTH: f64 = 14.0;
static ARROW_HEIGHT: f64 = 16.0;

fn venn3(id: usize, x: f64, y: f64, regions: u8) -> String {
    let r = 30;
    let a_cx = -17.3205 + x;
    let a_cy = 10.0 + y + FRAME_OFFSET;
    let b_cx = 0.0 + x;
    let b_cy = -20.0 + y + FRAME_OFFSET;
    let c_cx = 17.3205 + x;
    let c_cy = 10.0 + y + FRAME_OFFSET;
    let a_tx = -25.9807 + x;
    let a_ty = 15.0 + y + FRAME_OFFSET;
    let b_tx = -0.0 + x;
    let b_ty = -30.0 + y + FRAME_OFFSET;
    let c_tx = 25.9807 + x;
    let c_ty = 15.0 + y + FRAME_OFFSET;
    let rect_x = x - (BOX_WIDTH / 2.0);
    let rect_y = y - (BOX_HEIGHT / 2.0);

    return format!(
        r###"
          <!-- A -->
          <clipPath id="clip_{}_a">
            <circle cx="{}" cy="{}" r="{}"/>
          </clipPath>

          <!-- B -->
          <clipPath id="clip_{}_b">
            <circle cx="{}" cy="{}" r="{}"/>
          </clipPath>

          <!-- C -->
          <clipPath id="clip_{}_c">
            <circle cx="{}" cy="{}" r="{}"/>
          </clipPath>

          <!-- A intersect B -->
          <clipPath id="clip_{}_aib" clip-path="url('#clip_{}_a')">
            <circle cx="{}" cy="{}" r="{}"/>
          </clipPath>

          <!-- A intersect C -->
          <clipPath id="clip_{}_aic" clip-path="url('#clip_{}_a')">
            <circle cx="{}" cy="{}" r="{}"/>
          </clipPath>

          <!-- B intersect C -->
          <clipPath id="clip_{}_bic" clip-path="url('#clip_{}_c')">
            <circle cx="{}" cy="{}" r="{}"/>
          </clipPath>

          <!-- A intersect B intersect C -->
          <clipPath id="clip_{}_aibic" clip-path="url('#clip_{}_aib')">
            <circle cx="{}" cy="{}" r="{}"/>
          </clipPath>

          <!-- a intersect B intersect C -->
          <mask id="mask_{}_aibic" >
            <rect x="{}" y="{}" width="{}" height="{}" style="fill: white;"/>
            <rect clip-path="url('#clip_{}_aibic')" x="{}" y="{}" width="{}" height="{}" style="fill: black;"/>
          </mask>

          <!-- A union B -->
          <mask id="mask_{}_aub">
            <rect x="{}" y="{}" width="{}" height="{}" style="fill: white;"/>
            <circle cx="{}" cy="{}" r="{}" style="fill: black;"/>
            <circle cx="{}" cy="{}" r="{}" style="fill: black;"/>
          </mask>

          <!-- A union C -->
          <mask id="mask_{}_auc">
            <rect x="{}" y="{}" width="{}" height="{}" style="fill: white;"/>
            <circle cx="{}" cy="{}" r="{}" style="fill: black;"/>
            <circle cx="{}" cy="{}" r="{}" style="fill: black;"/>
          </mask>

          <!-- B union C -->
          <mask id="mask_{}_buc">
            <rect x="{}" y="{}" width="{}" height="{}" style="fill: white;"/>
            <circle cx="{}" cy="{}" r="{}" style="fill: black;"/>
            <circle cx="{}" cy="{}" r="{}" style="fill: black;"/>
          </mask>

          <!-- just A -->
          <circle clip-path="url('#clip_{}_a')" mask="url(#mask_{}_buc)" cx="{}" cy="{}" r="{}"{}/>
          <!-- just B -->
          <circle clip-path="url('#clip_{}_b')" mask="url(#mask_{}_auc)" cx="{}" cy="{}" r="{}"{}/>
          <!-- just C -->
          <circle clip-path="url('#clip_{}_c')" mask="url(#mask_{}_aub)" cx="{}" cy="{}" r="{}"{}/>
          <!-- A intersect B without C -->
          <circle clip-path="url('#clip_{}_aib')" mask="url(#mask_{}_aibic)" cx="{}" cy="{}" r="{}"{}/>
          <!-- A intersect C without B -->
          <circle clip-path="url('#clip_{}_aic')" mask="url(#mask_{}_aibic)" cx="{}" cy="{}" r="{}"{}/>
          <!-- B intersect C without A -->
          <circle clip-path="url('#clip_{}_bic')" mask="url(#mask_{}_aibic)" cx="{}" cy="{}" r="{}"{}/>
          <!-- A intersect B intersect C -->
          <circle clip-path="url('#clip_{}_aibic')" cx="{}" cy="{}" r="{}"{}/>

          <circle cx="{}" cy="{}" r="{}"/>
          <text x="{}" y="{}">A</text>
          <circle cx="{}" cy="{}" r="{}"/>
          <text x="{}" y="{}">B</text>
          <circle cx="{}" cy="{}" r="{}"/>
          <text x="{}" y="{}">C</text>

          <rect x="{}" y="{}" width="{}" height="{}" class="venn_frame"/>"###,
        id, a_cx, a_cy, r,
        id, b_cx, b_cy, r,
        id, c_cx, c_cy, r,
        id, id, b_cx, b_cy, r,
        id, id, c_cx, c_cy, r,
        id, id, b_cx, b_cy, r,
        id, id, c_cx, c_cy, r,
        id, rect_x, rect_y, BOX_WIDTH, BOX_HEIGHT, id, rect_x, rect_y, BOX_WIDTH, BOX_HEIGHT,
        id, rect_x, rect_y, BOX_WIDTH, BOX_HEIGHT, a_cx, a_cy, r, b_cx, b_cy, r,
        id, rect_x, rect_y, BOX_WIDTH, BOX_HEIGHT, a_cx, a_cy, r, c_cx, c_cy, r,
        id, rect_x, rect_y, BOX_WIDTH, BOX_HEIGHT, b_cx, b_cy, r, c_cx, c_cy, r,
        id, id, x, y, (BOX_WIDTH / 2.0) - 2.0, if (regions & 0b1000000) != 0 {r###"class="venn_yay""###} else {""},
        id, id, x, y, (BOX_WIDTH / 2.0) - 2.0, if (regions & 0b0000100) != 0 {r###"class="venn_yay""###} else {""},
        id, id, x, y, (BOX_WIDTH / 2.0) - 2.0, if (regions & 0b0000001) != 0 {r###"class="venn_yay""###} else {""},
        id, id, x, y, (BOX_WIDTH / 2.0) - 2.0, if (regions & 0b0100000) != 0 {r###"class="venn_yay""###} else {""},
        id, id, x, y, (BOX_WIDTH / 2.0) - 2.0, if (regions & 0b0010000) != 0 {r###"class="venn_yay""###} else {""},
        id, id, x, y, (BOX_WIDTH / 2.0) - 2.0, if (regions & 0b0000010) != 0 {r###"class="venn_yay""###} else {""},
        id, x, y, (BOX_WIDTH / 2.0) - 2.0, if (regions & 0b0001000) != 0 {r###"class="venn_yay""###} else {""},
        a_cx, a_cy, r, a_tx, a_ty,
        b_cx, b_cy, r, b_tx, b_ty,
        c_cx, c_cy, r, c_tx, c_ty,
        rect_x, rect_y, BOX_WIDTH, BOX_HEIGHT
    );
}

fn venn2(id: usize, x: f64, y: f64, regions: u8) -> String {
    let r = 30;
    let a_cx = -15.0 + x;
    let a_cy = y;
    let b_cx = 15.0 + x;
    let b_cy = y;
    let a_tx = -25.0 + x;
    let a_ty = y;
    let b_tx = 25.0 + x;
    let b_ty = y;
    let rect_x = x - (BOX_WIDTH / 2.0);
    let rect_y = y - (BOX_HEIGHT2 / 2.0);

    return format!(
        r###"
          <!-- A -->
          <clipPath id="clip_{}_a">
            <circle cx="{}" cy="{}" r="{}"/>
          </clipPath>

          <!-- B -->
          <clipPath id="clip_{}_b">
            <circle cx="{}" cy="{}" r="{}"/>
          </clipPath>

          <!-- A intersect B -->
          <clipPath id="clip_{}_aib" clip-path="url('#clip_{}_a')">
            <circle cx="{}" cy="{}" r="{}"/>
          </clipPath>

          <!-- A -->
          <mask id="mask_{}_a">
            <rect x="{}" y="{}" width="{}" height="{}" style="fill: white;"/>
            <circle cx="{}" cy="{}" r="{}" style="fill: black;"/>
          </mask>

          <!-- B -->
          <mask id="mask_{}_b">
            <rect x="{}" y="{}" width="{}" height="{}" style="fill: white;"/>
            <circle cx="{}" cy="{}" r="{}" style="fill: black;"/>
          </mask>

          <!-- just A -->
          <circle clip-path="url('#clip_{}_a')" mask="url(#mask_{}_b)" cx="{}" cy="{}" r="{}"{}/>
          <!-- just B -->
          <circle clip-path="url('#clip_{}_b')" mask="url(#mask_{}_a)" cx="{}" cy="{}" r="{}"{}/>
          <!-- A intersect B -->
          <circle clip-path="url('#clip_{}_aib')" cx="{}" cy="{}" r="{}"{}/>

          <circle cx="{}" cy="{}" r="{}"/>
          <text x="{}" y="{}">A</text>
          <circle cx="{}" cy="{}" r="{}"/>
          <text x="{}" y="{}">B</text>

          <rect x="{}" y="{}" width="{}" height="{}" class="venn_frame"/>"###,
        id, a_cx, a_cy, r,
        id, b_cx, b_cy, r,
        id, id, b_cx, b_cy, r,
        id, rect_x, rect_y, BOX_WIDTH, BOX_HEIGHT, a_cx, a_cy, r,
        id, rect_x, rect_y, BOX_WIDTH, BOX_HEIGHT, b_cx, b_cy, r,
        id, id, x, y, (BOX_WIDTH / 2.0) - 2.0, if (regions & 0b1000000) != 0 {r###"class="venn_yay""###} else {""},
        id, id, x, y, (BOX_WIDTH / 2.0) - 2.0, if (regions & 0b0000100) != 0 {r###"class="venn_yay""###} else {""},
        id, x, y, (BOX_WIDTH / 2.0) - 2.0, if (regions & 0b0100000) != 0 {r###"class="venn_yay""###} else {""},
        a_cx, a_cy, r, a_tx, a_ty,
        b_cx, b_cy, r, b_tx, b_ty,
        rect_x, rect_y, BOX_WIDTH, BOX_HEIGHT2,
    );
}

// https://www.reingold.co/tidier-drawings.pdf

use crate::Out;
use core::ptr::null_mut;

#[derive(Debug)]
pub struct Node {
    pub llink: *mut Node,
    pub rlink: *mut Node,
    pub xcoord: f64,
    pub ycoord: usize,
    offset: f64,
    thread: bool,
}

impl Node {
    pub fn new_leaf() -> Self {
        Node {
            llink: null_mut(),
            rlink: null_mut(),
            xcoord: 0.0,
            ycoord: 0,
            offset: 0.0,
            thread: false,
        }
    }

    pub fn new_l(l: *mut Node) -> Self {
        let mut n = Self::new_leaf();
        n.llink = l;
        return n;
    }

    pub fn new_r(r: *mut Node) -> Self {
        let mut n = Self::new_leaf();
        n.rlink = r;
        return n;
    }

    pub fn new_lr(l: *mut Node, r: *mut Node) -> Self {
        let mut n = Self::new_leaf();
        n.llink = l;
        n.rlink = r;
        return n;
    }

    pub fn free(s: *mut Node) {
        unsafe {
            if !(*s).llink.is_null() {
                Node::free((*s).llink);
            }
            if !(*s).rlink.is_null() {
                Node::free((*s).rlink);
            }
            let _freed_automatically = Box::from_raw(s);
        }
    }

    pub fn dimensions(&self) -> Dimensions {
        let mut d = Dimensions {
            max_y: 0,
            min_x: 0.0,
            max_x: 0.0,
            count: 0,
        };
        dimensions_(self, &mut d);
        return d;
    }
}

fn dimensions_(n: *const Node, d: &mut Dimensions) {
    unsafe {
        d.max_y = max(d.max_y, (*n).ycoord);
        d.min_x = f64::min(d.min_x, (*n).xcoord);
        d.max_x = f64::max(d.max_x, (*n).xcoord);
        d.count += 1;

        if !(*n).llink.is_null() {
            dimensions_((*n).llink, d);
        }
        if !(*n).rlink.is_null() {
            dimensions_((*n).rlink, d);
        }
    }
}

pub fn setup(t: *mut Node, minsep: f64) {
    let level = 0;
    let mut rmost = Extreme::new();
    let mut lmost = Extreme::new();
    return setup_(t, level, &mut rmost, &mut lmost, minsep);
}

fn setup_(t: *mut Node, level: usize, rmost: *mut Extreme, lmost: *mut Extreme, minsep: f64) {
    unsafe {
        if t.is_null() {
            (*lmost).lev = None;
            (*rmost).lev = None;
        } else {
            (*t).ycoord = level;
            let mut l = (*t).llink;
            let mut r = (*t).rlink;
            let mut lr = Extreme::new();
            let mut ll = Extreme::new();
            let mut rr = Extreme::new();
            let mut rl = Extreme::new();

            setup_(l, level + 1, &mut lr, &mut ll, minsep);
            setup_(r, level + 1, &mut rr, &mut rl, minsep);

            if r.is_null() && l.is_null() {
                (*rmost).addr = t;
                (*lmost).addr = t;
                (*rmost).lev = Some(level);
                (*lmost).lev = Some(level);
                (*rmost).off = 0.0;
                (*lmost).off = 0.0;
                (*t).offset = 0.0;
            } else {
                let mut cursep = minsep;
                let mut rootsep = minsep;
                let mut loffsum = 0.0;
                let mut roffsum = 0.0;

                while !l.is_null() && !r.is_null() {
                    if cursep < minsep {
                        rootsep += minsep - cursep;
                        cursep = minsep;
                    }

                    if !(*l).rlink.is_null() {
                        loffsum += (*l).offset;
                        cursep -= (*l).offset;
                        l = (*l).rlink;
                    } else {
                        loffsum -= (*l).offset;
                        cursep += (*l).offset;
                        l = (*l).llink;
                    }
                    if !(*r).llink.is_null() {
                        roffsum -= (*r).offset;
                        cursep -= (*r).offset;
                        r = (*r).llink;
                    } else {
                        roffsum += (*r).offset;
                        cursep += (*r).offset;
                        r = (*r).rlink;
                    }
                }

                (*t).offset = (rootsep + 1.0) / 2.0;
                loffsum -= (*t).offset;
                roffsum += (*t).offset;

                if (rl.lev > ll.lev) || (*t).llink.is_null() {
                    *lmost = rl;
                    (*lmost).off += (*t).offset;
                } else {
                    *lmost = ll;
                    (*lmost).off -= (*t).offset;
                }
                if (lr.lev > rr.lev) || (*t).rlink.is_null() {
                    *rmost = lr;
                    (*rmost).off -= (*t).offset;
                } else {
                    *rmost = rr;
                    (*rmost).off += (*t).offset;
                }

                if !l.is_null() && (l != (*t).llink) {
                    (*rr.addr).thread = true;
                    (*rr.addr).offset = f64::abs((rr.off + (*t).offset) - loffsum);
                    if loffsum - (*t).offset <= rr.off {
                        (*rr.addr).llink = l;
                    } else {
                        (*rr.addr).rlink = l;
                    }
                } else if !r.is_null() && (r != (*t).rlink) {
                    (*ll.addr).thread = true;
                    (*ll.addr).offset = f64::abs((ll.off - (*t).offset) - roffsum);
                    if roffsum + (*t).offset >= ll.off {
                        (*ll.addr).rlink = r;
                    } else {
                        (*ll.addr).llink = r;
                    }
                }
            }
        }
    }
}

pub fn petrify(t: *mut Node) {
    petrify_(t, 0.0);
}

fn petrify_(t: *mut Node, xpos: f64) {
    unsafe {
        if !t.is_null() {
            (*t).xcoord = xpos;
            if (*t).thread {
                (*t).thread = false;
                (*t).rlink = null_mut();
                (*t).llink = null_mut();
            }
            petrify_((*t).llink, xpos - (*t).offset);
            petrify_((*t).rlink, xpos + (*t).offset);
        }
    }
}

#[derive(Clone, Copy)]
pub struct Extreme {
    addr: *mut Node,
    off: f64,
    lev: Option<usize>,
}

impl Extreme {
    pub fn new() -> Self {
        Extreme {
            addr: null_mut(),
            off: 0.0,
            lev: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test0() {
        let mut l0 = Node::new_leaf();
        let mut l1 = Node::new_leaf();
        let mut l2 = Node::new_leaf();
        let mut i0 = Node::new_lr(&mut l0, &mut l1);
        let mut r = Node::new_lr(&mut i0, &mut l2);

        setup(&mut r, 1.0);
        petrify(&mut r);
        assert_eq!(r.xcoord, 0.0);
        assert_eq!(i0.xcoord, -1.0);
        assert_eq!(l0.xcoord, -2.0);
        assert_eq!(l1.xcoord, 0.0);
        assert_eq!(l2.xcoord, 1.0);
    }

    #[test]
    fn test1() {
        let mut l0 = Node::new_leaf();
        let mut l1 = Node::new_leaf();
        let mut l2 = Node::new_leaf();
        let mut i0 = Node::new_lr(&mut l1, &mut l2);
        let mut r = Node::new_lr(&mut l0, &mut i0);

        setup(&mut r, 1.0);
        petrify(&mut r);
        assert_eq!(r.xcoord, 0.0);
        assert_eq!(i0.xcoord, 1.0);
        assert_eq!(l0.xcoord, -1.0);
        assert_eq!(l1.xcoord, 0.0);
        assert_eq!(l2.xcoord, 2.0);
    }

    #[test]
    fn test2() {
        let mut l0 = Node::new_leaf();
        let mut l1 = Node::new_leaf();
        let mut l2 = Node::new_leaf();
        let mut l3 = Node::new_leaf();
        let mut i0 = Node::new_lr(&mut l0, &mut l1);
        let mut i1 = Node::new_lr(&mut l2, &mut l3);
        let mut r = Node::new_lr(&mut i0, &mut i1);

        setup(&mut r, 1.0);
        petrify(&mut r);
        assert_eq!(r.xcoord, 0.0);
        assert_eq!(i0.xcoord, -2.0);
        assert_eq!(i1.xcoord, 2.0);
        assert_eq!(l0.xcoord, -3.0);
        assert_eq!(l1.xcoord, -1.0);
        assert_eq!(l2.xcoord, 1.0);
        assert_eq!(l3.xcoord, 3.0);
    }
}
