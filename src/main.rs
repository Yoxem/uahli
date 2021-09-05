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
/// - x: the x length of moving in px.
/// - y: the y length of moving in px
/// - x_offset: the x moving from the baseline in px
/// - y_offset: the y moving from the baseline in px
/// 
#[derive(Debug)]
struct BoxCoodInfo {
    text: String,
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

/// a Div text layout block. unit: px
/// - x: x-axis in px (from left)
/// - y: y-axis in px (from top)
/// - width: Div width in px
/// - height: Div height in px
/// - lineskip: the skip between the baseline of 2 lines in px
/// - direction: Rtl, Ltr, Btt, Ttb
/// - color: #ffffff -like hex html color code
#[derive(Debug)]
struct Div{
    x: f64,
    y: f64,
    width:f64,
    height: f64,
    lineskip: f64,
    direction: harfbuzz_rs::Direction,
    color: String
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

        let mut box_cood = BoxCoodInfo{text: text.to_string(), x: 0.0, y  : 0.0, x_offset: 0.0, y_offset : 0.0};

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

            // convert pt to px
            box_cood.x *= 0.8;
            box_cood.y *= 0.8;
            box_cood.x_offset *= 0.8;
            box_cood.y_offset *= 0.8;

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
fn layout_text(text : &str, mut font_struct: &FontStruct, x : f64, y: f64,color: &str, direction: harfbuzz_rs::Direction, mut canva: &cairo::Context)->Option<BoxCoodInfo>{
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


fn greedy_typesetting(box_coord_vec : Vec<Option<BoxCoodInfo>>, block: Div, font: FontStruct, cxt : &cairo::Context){
    let mut current_x = block.x;
    let mut current_y = block.y;

    for i in &box_coord_vec{
        match i {
            Some(inner) =>{
                if (current_x + inner.x) <= block.x + block.width {
                    layout_text(&(inner.text), &font,  current_x, current_y, &(block.color) ,block.direction , &cxt);
                    current_x += inner.x
                // try to add a new line
                }else{
                    current_x = block.x;
                    current_y += block.lineskip;
                    // if beneath the margin of the botton, don't layout it and break
                    if current_y > block.height{
                        break;
                    }
                    else{
                        layout_text(&(inner.text), &font,  current_x, current_y, &(block.color) ,block.direction , &cxt);
                        current_x += inner.x
                    }

                }
            }
            None    => println!("The text segment can't be layouted."),
        }
    }

}

fn main(){
    let font_pt = 20;

    /*let font_name = "Amstelvar";
    let font_style = "Italic";*/

    let font_name = "AR PL UMing TW";
    let font_style = "Bold";

    const PDF_WIDTH_IN_PX : f64 = 595.0;
    const PDF_HEIGHT_IN_PX : f64 = 842.0;
    let pdf_path = "/tmp/a.pdf";

    let mut regex_pattern1 = r"([^\s\p{Bopomofo}\p{Han}\p{Hangul}\p{Hiragana}\p{Katakana}。，、；：「」『』（）？！─……《》〈〉．～～゠‥｛｝［］〔〕〘〙〈〉《》【】〖〗※〳〵〴〲〱〽〃]{1,}|".to_string();
    let regex_pattern2 = r"[\p{Bopomofo}\p{Han}\p{Hangul}\p{Hiragana}\p{Katakana}。，、；：「」『』（）？！─……《》〈〉．～～゠‥｛｝［］〔〕〘〙〈〉《》【】〖〗※〳〵〴〲〱〽〃]|──|〴〵|〳〵)";
    regex_pattern1.push_str(&regex_pattern2);
    let regex_pattern = Regex::new(&regex_pattern1).unwrap();
    //let input_text = "我kā lí講這件——代誌彼は아버지 감사합니다といいます。It's true. happier. Ta̍k-ke. ٱلسَّلَامُ عَلَيْكُمْ שָׁלוֹם עֲלֵיכֶם";
    let input_text = "望月峯頭白露滋，南飛烏鵲怨無枝；不知消瘦嫦娥影，還得娟娟似舊時？題望月峯——梁啟超。「旗中黃虎尚如生，國建共和怎不成。天與台灣原獨立，我疑記載欠分明。」「唉！寂寞的人生 / 寂寞得 / 似沙漠上的孤客 / 這句經誰說過的話 / 忽回到我善忘的記憶 / 在紛擾擾的人世間 / 我儘在孤獨蕭瑟 / 像徘徊在沙漠中 / 找不到行過人的蹤跡。」——賴和。吳濁流：「 永夜　永夜　沒有星光　沒有月亮　沒有詩聲　沒有歌唱　萬籟俱寂　天地無分　黑暗　黑暗　黑暗」——吳濁流";



    let mut input_text_vec = vec!();

    let block = Div{x:100.0,y:100.0,width:450.0, height: 250.0, lineskip: 30.0, direction: harfbuzz_rs::Direction::Ltr, color: "#198964".to_string()};


    let mut text : String;
    for cap in regex_pattern.captures_iter(input_text){
        let text = cap[0].to_string().clone();
        
        input_text_vec.push(text);
    }

    println!("{:?}", input_text_vec);

    let mut font_struct1 =  FontStruct{size:font_pt, name:font_name, style:font_style, variations : &[Variation::new(b"wght", 200.0),
         Variation::new(b"wdth", 20.0)], features : &[]};

    let box_coord_vec : Vec<Option<BoxCoodInfo>> = input_text_vec.into_iter().map(|x| get_box_cood_info(&x, font_struct1.name, font_struct1.style, font_struct1.size, harfbuzz_rs::Direction::Ltr, &[], &[])).collect();

    println!("{:?}", box_coord_vec);



    let surface = cairo::PdfSurface::new(PDF_WIDTH_IN_PX, PDF_HEIGHT_IN_PX, pdf_path).expect("Couldn’t create surface"); // A4 size

    let cxt = cairo::Context::new(&surface).expect("running error");

    


        cxt.set_source_rgba(0.8, 1.0, 1.0, 0.5); // 設定顏色
        cxt.paint().ok();// 設定背景顏色

        cxt.set_source_rgba(0.0, 0.0, 1.0, 1.0); // 設定顏色
        println!("{:?}",cxt.source());

        
        let font_struct2 =  FontStruct{size:30, name:"Noto Sans CJK TC", style:"Bold", variations : &[], features : &[]};

        let font_struct3 =  FontStruct{size:30, name:"Noto Nastaliq Urdu", style:"Bold", variations : &[], features : &[]};

        //layout_text("Tá grá agam duit", font_struct1,  100.0, 100.0,"#198964",harfbuzz_rs::Direction::Ltr, &cxt);

        layout_text("一寡詩歌", &font_struct2,  50.0, 200.0,"#0000ff",harfbuzz_rs::Direction::Ltr, &cxt);

        //layout_text("انا احبك ", &font_struct3,  100.0, 300.0,"#198964",harfbuzz_rs::Direction::Rtl, &cxt);
        // println!("{:?}", result);

        greedy_typesetting(box_coord_vec, block, font_struct1, &cxt);
}
