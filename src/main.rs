extern crate cairo;
extern crate fontconfig;
extern crate pango;
use std::convert::TryInto;
use std::str;
use harfbuzz_rs::*;
use pangocairo;
use pangocairo::prelude::FontMapExt;
use regex::Regex;


///
/// 
/// Get the infomation of a coodinrate of a box (text slice).
/// - x: the x length of moving.
/// - y: the y length of moving
/// - x_offset: the x moving from the baseline
/// - y_offset: the y moving from the baseline
/// 
#[derive(Debug)]
struct BoxCoodInfo {
    x: f64,
    y: f64,
    x_offset: f64,
    y_offset: f64
}
///
/// RgbColor: storing 0~255 rgb color code
/// - red: 0 ~ 255
/// - green: 0 ~ 255
/// - blue: 0 ~ 255
#[derive(Debug)]
struct RgbColor{
    red: u32, 
    green: u32,
    blue: u32
}
/// 
/// The structure storing a font and its attribution.
/// - name : font name . eg. "FreeSans"
/// - style : font style. eg. "Italic"
/// - size : in pt. eg. "32" for 32pt
/// - variations. variation of a opentype font. eg. `vec![Variation::new(b"wght", 800.0);]`
/// - features. features of a opentype font. eg. `vec![Feature::new(b"calt", 1, 0..)]`
/// 
#[derive(Debug)]
struct FontStruct<'a> {
    name: &'a str,
    style: &'a str,
    size: u32,
    variations : &'a  [Variation],
    features: &'a  [Feature]
}



/// get the cood infomation of the input box.
/// - text : the render text
/// - font_name: "FreeSans", etc
/// - font_style: "Bold", etc
/// - font_size_pt: 16, etc
/// - direction: Ltr, Rtl, etc,
/// - variations: Opentype variation axis list
/// - features: Opentype feature list
fn get_box_cood_info(text : &str, font_name : &str, font_style : &str, font_size_pt : u32, 
    direction : harfbuzz_rs::Direction, variations : &[Variation], features: & [Feature])
    ->  Option<BoxCoodInfo>  {
        // let font_combined = format!("{} {}", font_name, font_style);

        let fc = fontconfig::Fontconfig::new()?;
        let font = fc.find(font_name, Some(font_style))?;
        let path = font.path.to_str()?;
        // println!("{}", path);;

        let index = 0; //< face index in the font file
        let face = Face::from_file(path, index).ok()?;

        let mut font = Font::new(face); // setting the font

        if !variations.is_empty(){
            font.set_variations(&variations);
        }

        font.set_scale((font_size_pt*64).try_into().unwrap(), (font_size_pt*64).try_into().unwrap()); // setting the pt size

        let buffer = UnicodeBuffer::new().set_direction(direction).add_str(text);

        // shape the text box
        let output = shape(&font, buffer, &features);

        // The results of the shaping operation are stored in the `output` buffer.
        let positions = output.get_glyph_positions();
        let infos = output.get_glyph_infos();

        
        assert_eq!(positions.len(), infos.len());

        let mut box_cood = BoxCoodInfo{x: 0.0, y  : 0.0, x_offset: 0.0, y_offset : 0.0};

        for position in positions {
            /*let gid = info.codepoint;
            let cluster = info.cluster;
            let glyph_name = Font::get_glyph_name(&font, gid);*/

            let x_advance = (position.x_advance) as f64/64.0;
            let y_advance = (position.y_advance) as f64/64.0;
            let x_offset = (position.x_offset) as f64/64.0;
            let y_offset = (position.y_offset) as f64/64.0;
        
        

            // Here you would usually draw the glyphs.
            //println!("gid{:?}[{:?}]={:?}@{:?}:{:?},\t{:?}+{:?}", gid, glyph_name, cluster, current_x, current_y, x_offset, y_offset);

            // set the max_x(y)_advance as the box_cood.x(y)_advance
            if box_cood.x_offset<x_offset{
                box_cood.x_offset = x_offset
            }
            if box_cood.y_offset<y_offset{
                box_cood.y_offset = y_offset
            }

            box_cood.x += x_advance;
            box_cood.y += y_advance;
        }
    
    return Some(box_cood);

}

///
///
/// converting font variant list to string. eg.
/// `font_variant_list_to_string(vec![Variation::new(b"wght", 800.0), Variation::new(b"wdth", 50.0)]);`
/// ==> `"wght=800.0,wdth=50.0,"`
/// - vars: variation list
fn font_variant_list_to_string(vars : &[Variation]) -> String{
    let mut string : String;
    string  = "".to_string();
    for i in vars{
        let var_combined = format!("{}={},", i.tag(), i.value());
        string.push_str(&var_combined);
    }
    return string;

}

///
/// 
/// convert hex color code to rgb 256 number. eg.
/// #ffffff -> RgbColor{red:256, green:256, blue:256}
/// - hex : the hex color code to be input
fn hex_color_code_to_int(hex : &str)->Option<RgbColor>{
    let pattern = Regex::new(r"#(?P<r>[0-9a-fA-F]{2})(?P<g>[0-9a-fA-F]{2})(?P<b>[0-9a-fA-F]{2})").unwrap();
    let caps = pattern.captures(hex).unwrap();
    let mut rgb = RgbColor{red:0, green:0, blue:0};
    let r = caps.name("r")?;
    rgb.red = u32::from_str_radix(r.as_str(),16).ok()?;
    let g = caps.name("g")?;
    rgb.green = u32::from_str_radix(g.as_str(),16).ok()?;
    let b = caps.name("b")?;
    rgb.blue = u32::from_str_radix(b.as_str(),16).ok()?;

    return Some(rgb);

}

///
/// 
/// show `text` in `canva` at `x` and `y` with `font_struct`
/// text : the text to be rendered.
/// font_sruct: font and its attributes
/// x : x-axis coord in px.
/// y : y-axis coord in px.
/// color : hex color `#000000`, etc
/// canva : cairo canvas
/// return box_cood if it runs successfully.
fn layout_text(text : &str, mut font_struct: FontStruct, x : f64, y: f64,color: &str, direction: harfbuzz_rs::Direction, mut canva: &cairo::Context)->Option<BoxCoodInfo>{
    let fontmap = pangocairo::FontMap::default().unwrap();
    let font_combined = format!("{} {}", font_struct.name, font_struct.style);

    let mut font_with_style = pango::FontDescription::from_string(&font_combined);
   //  pango_font_style.set_absolute_size((font_struct.size * 1024).into());
   font_with_style.set_variations(&font_variant_list_to_string(font_struct.variations));
    let pango_cxt = pango::Context::new();
    // println!("{:?}", pango::AttrType::Fallback);

    let _pango_cairo_font = fontmap.load_font(&pango_cxt, &font_with_style);

    let box_cood = get_box_cood_info(text, font_struct.name, font_struct.style, font_struct.size, direction, font_struct.variations, &[])?;

    pango_cxt.set_font_map(&fontmap);
    let pango_layout = pango::Layout::new(&pango_cxt);
    pango_layout.set_font_description(Some(&font_with_style));
    pango_layout.set_text(text);

    // setting the color
    canva.save();
    let color_rgb = hex_color_code_to_int(color)?;
    
    canva.set_source_rgb(color_rgb.red as f64/256.0, color_rgb.green as f64/256.0, color_rgb.blue as f64/256.0);


    canva.move_to(x, y);
    pangocairo::show_layout(&canva, &pango_layout);

    canva.restore();
    canva.move_to(0.0, 0.0);

    return Some(box_cood);
}

fn main(){
    let font_pt = 20;

    let font_name = "Amstelvar";
    let font_style = "Italic";


    const PDF_WIDTH_IN_PX : f64 = 595.0;
    const PDF_HEIGHT_IN_PX : f64 = 842.0;
    let pdf_path = "/tmp/a.pdf";


    let surface = cairo::PdfSurface::new(PDF_WIDTH_IN_PX, PDF_HEIGHT_IN_PX, pdf_path).expect("Couldn’t create surface"); // A4 size

    let cxt = cairo::Context::new(&surface).expect("running error");

    


        cxt.set_source_rgba(0.8, 1.0, 1.0, 0.5); // 設定顏色
        cxt.paint().ok();// 設定背景顏色

        cxt.set_source_rgba(0.0, 0.0, 1.0, 1.0); // 設定顏色
        println!("{:?}",cxt.source());

        let font_struct1 =  FontStruct{size:font_pt, name:font_name, style:font_style, variations : &[Variation::new(b"wght", 200.0),
         Variation::new(b"wdth", 20.0)], features : &[]};
        let font_struct2 =  FontStruct{size:30, name:"Noto Sans CJK TC", style:"Bold", variations : &[], features : &[]};

        let font_struct3 =  FontStruct{size:30, name:"Noto Nastaliq Urdu", style:"Bold", variations : &[], features : &[]};

        layout_text("Tá grá agam duit", font_struct1,  100.0, 100.0,"#198964",harfbuzz_rs::Direction::Ltr, &cxt);

        layout_text("我疼Lí", font_struct2,  100.0, 200.0,"#198964",harfbuzz_rs::Direction::Ltr, &cxt);

        layout_text("انا احبك ", font_struct3,  100.0, 300.0,"#198964",harfbuzz_rs::Direction::Rtl, &cxt);
        // println!("{:?}", result);
}
