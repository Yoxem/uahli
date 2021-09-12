extern crate cairo;
extern crate fontconfig;
extern crate pango;
use std::convert::TryInto;
use std::collections::HashMap;
use std::str;
use harfbuzz_rs::*;
use pangocairo;
use pangocairo::prelude::FontMapExt;
use regex::Regex;
use std::str::FromStr;



///
///
/// storing line data.
/// - content : the vec of the line content of Option<BoxCoodInfo>
/// - div_text_offset: the offset between div width and text width
#[derive(Debug)]
struct Line<'a>{
    content: Vec<&'a BoxCoodInfo>,
    div_text_offset: f64
}

pub struct MinRaggedLayouter {
    hashmap_cost: HashMap<u32, f64>,
    hashmap_route: HashMap<u32, u32>,
    words: String,
    path: Vec<u32>
    }

    /*
impl MinRaggedLayouter {
        ///
        /// counting total_cost of a line ragged cost.
        ///  - words : the words listed here.
        ///  - dest : destination (k)
        ///  - maxwidth: in pt.
        pub fn total(&mut self, words : Vec<Option<BoxCoodInfo>>, dest : u32, maxwidth : f64 ) -> f64 {

            
        }
    }*/

///
///
/// Get the infomation of a coodinrate of a box (text slice).
/// - text: the text of it.
/// - width: the x length of moving in pt.
/// - height: the y length of moving in pt
/// - x_offset: the x moving from the baseline in pt
/// - y_offset: the y moving from the baseline in pt
///
#[derive(Debug)]
struct BoxCoodInfo {
    text: String,
    width: f64,
    height: f64,
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

/// a Div text layout block. unit: pt
/// - x: x-axis in pt (from left)
/// - y: y-axis in pt (from top)
/// - width: Div width in pt
/// - height: Div height in pt
/// - lineskip: the skip between the baseline of 2 lines in pt
/// - direction: Rtl, Ltr, Btt, Ttb
/// - color: #ffffff -like hex html color code
#[derive(Debug)]
struct Div<'a>{
    x: f64,
    y: f64,
    width:f64,
    height: f64,
    lineskip: f64,
    language: &'a str,
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
fn get_box_cood_info(text : &str, font_name : &str, font_style : &str, font_size_pt : u32, language: &str,
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

        let hb_language = harfbuzz_rs::Language::from_str(language).unwrap();

        let mut buffer = UnicodeBuffer::new().set_direction(direction).set_language(hb_language).add_str(text);
        

        // shape the text box
        let output = shape(&font, buffer, &features);

        // The results of the shaping operation are stored in the `output` buffer.
        let positions = output.get_glyph_positions();
        let infos = output.get_glyph_infos();


        assert_eq!(positions.len(), infos.len());

        let mut box_cood = BoxCoodInfo{text: text.to_string(), width: 0.0, height  : 0.0, x_offset: 0.0, y_offset : 0.0};

        for position in positions{
 

            let x_advance = (position.x_advance) as f64/64.0;
            let y_advance = (position.y_advance) as f64/64.0;
            let x_offset = (position.x_offset) as f64/64.0;
            let y_offset = (position.y_offset) as f64/64.0;




            // set the max_x(y)_advance as the box_cood.x(y)_advance
            if box_cood.x_offset<x_offset{
                box_cood.x_offset = x_offset
            }
            if box_cood.y_offset<y_offset{
                box_cood.y_offset = y_offset
            }

            box_cood.width += x_advance;
            box_cood.height += y_advance;



        }

    // convert to pt
    box_cood.width;
    box_cood.height;
    box_cood.x_offset;
    box_cood.y_offset;

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
/// x : x-axis coord in pt.
/// y : y-axis coord in pt.
/// color : hex color `#000000`, etc
/// lang: string eg. "en", "zh", etc.
/// direction: harfbuzz_rs::Direction
/// canva : cairo canvas
/// return box_cood if it runs successfully.
fn layout_text(text : &str, mut font_struct: &FontStruct, x : f64, y: f64,color: &str, lang: &str, direction: harfbuzz_rs::Direction, mut canva: &cairo::Context)
->Option<()>
{
    let fontmap = pangocairo::FontMap::default().unwrap();

    // you have to multiply 0.75 or you'll get a bug.
    let font_combined = format!("{} {} {}", font_struct.name, font_struct.style, (font_struct.size as f64) * 0.75);

    let mut font_with_style = pango::FontDescription::from_string(&font_combined);
   //  pango_font_style.set_absolute_size((font_struct.size * 1024).into());
   font_with_style.set_variations(&font_variant_list_to_string(font_struct.variations));
    let pango_cxt = pango::Context::new();
    // println!("{:?}", pango::AttrType::Fallback);

    let _pango_cairo_font = fontmap.load_font(&pango_cxt, &font_with_style);

    //let box_cood = get_box_cood_info(text, font_struct.name, font_struct.style, font_struct.size, direction, font_struct.variations, &[])?;

    pango_cxt.set_language(&pango::language::Language::from_string(lang));

    pango_cxt.set_font_map(&fontmap);
    let pango_layout = pango::Layout::new(&pango_cxt);
    pango_layout.set_font_description(Some(&font_with_style));
    pango_layout.set_text(text);

    // setting the color
    canva.save().ok();
    let color_rgb = hex_color_code_to_int(color)?;

    canva.set_source_rgb(color_rgb.red as f64/256.0, color_rgb.green as f64/256.0, color_rgb.blue as f64/256.0);


    canva.move_to(x, y);
    pangocairo::show_layout(&canva, &pango_layout);

    canva.restore().ok();
    canva.move_to(0.0, 0.0);

    return Some(());
}



///
/// typesetting for greedy algorithm using unragged.
/// for arguments, see `greedy_typesetting`
fn greedy_typesetting(box_coord_vec : Vec<Option<BoxCoodInfo>>, block: Div, font: FontStruct, cxt : &cairo::Context, ragged : bool){

    //get the space width.
    let mut space_width = 0.0;

    let space_box_cood = get_box_cood_info(" ", font.name, font.style, font.size, block.language, block.direction,  font.variations, font.features);
    match space_box_cood {
        Some(inner) =>{
            space_width = inner.width;
        }
        None=>println!("The space width can't be defined. Set it to 0.")
    }
    let mut lines = vec![];  // store lines
    let mut line =  vec![];  // store a line

    let mut current_x = block.x;
    let mut current_y = block.y;

    let mut div_txt_offset = 0.0; // the offset between div width and text width

    let mut is_overflowed = false;

    for i in &box_coord_vec{
        match i {
            Some(inner) =>{
                let mut inner_width = inner.width;
                if Regex::new(r"[ \t\n]+").unwrap().is_match(&(inner.text)){
                    inner_width = space_width;
                }

                if (current_x + inner_width) <= block.x + block.width {
                    line.push(inner);

                    // if inner is not space, set the div_txt_offset
                    if !is_space(&inner.text){
                        div_txt_offset = block.x + block.width - (current_x + inner.width); 
                    }
                    current_x += inner_width;
                // try to add a new line
                }else{


                    current_x = block.x;
                    current_y += block.lineskip;

                    let div_txt_offset_clone = div_txt_offset.clone();

                    let line_clone  = line.clone();
                    let line_content = Line{
                                        content: line_clone,
                                        div_text_offset: div_txt_offset_clone};

                    lines.push(line_content);


                    // if beneath the margin of the botton, don't layout it and break
                    if current_y > block.y + block.height{
                        is_overflowed = true;
                        break;
                    }
                    else{
                        /*println!("{:?}", space_width);
                        println!("{:?}", div_txt_offset);
                        println!("{:?}", block.x + block.width);*/

                        line = vec![];
                        div_txt_offset = 0.0;

                        // if it's non space, add it.
                        if !Regex::new(r"[ \t\n]+").unwrap().is_match(&(inner.text)){
                            line.push(inner);
                            current_x += inner_width;
                        }

                    }

                }
            }
            None    => println!("The text segment can't be layouted."),
        }


    }

    // if it's not overflowed,  push the last line.

    if !is_overflowed{
        let div_txt_offset_clone = div_txt_offset.clone();

        let line_clone  = line.clone();
        let line_content = Line{
                                    content: line_clone,
                                    div_text_offset: div_txt_offset_clone};

        lines.push(line_content);

    }else{

    }

    // layout the characters
    if ragged == true{
        current_y = block.y;
        current_x = block.x;
        for i in 0..lines.len(){
            for j in 0..lines[i].content.len(){
                let con = lines[i].content[j];

                let mut content_width = con.width;
                // if it's space, set it to space_width.
                if is_space(&(con.text)){
                    content_width = space_width;
                }


                layout_text(&(con.text), &font,  current_x, current_y, &(block.color) ,block.language, block.direction , &cxt);
                current_x += content_width;
            }
            current_y += block.lineskip;
            current_x = block.x;
        }

    // unragged.
    }else{

        current_y = block.y;
        current_x = block.x;
        for i in 0..lines.len(){

            let mut line_word_len_without_space = 0;

            // count line word actually len (without spaces)
            for j in 0..lines[i].content.len(){
                if !is_space(&(lines[i].content[j].text)){
                    line_word_len_without_space += 1;
                }
            }
            // non last line
            if (i < lines.len() - 1) || (is_overflowed) {


                let line_space_width = space_width + lines[i].div_text_offset / (line_word_len_without_space as f64 - 1.0);

                for j in 0..lines[i].content.len(){
                        let con = lines[i].content[j];


                    if is_space(&(con.text)){
                        current_x += line_space_width;
                    }else{

                        layout_text(&(con.text), &font,  current_x, current_y, &(block.color) , block.language, block.direction , &cxt);
                        current_x += con.width;
                    }
                }

                current_y += block.lineskip;
                current_x = block.x;
            }
            // last line and if it's not overflowed
            else{
                for j in 0..lines[i].content.len(){
                    let con = lines[i].content[j];

                    let mut content_width = con.width;
                    // if it's space, set it to space_width.
                    if is_space(&(con.text)){
                        content_width = space_width;
                    }


                    layout_text(&(con.text), &font,  current_x, current_y, &(block.color) , block.language, block.direction , &cxt);
                    current_x += content_width;
                }
            }

        }


    }
}

/// check if it's a space of not.
///
///
fn is_space(txt : &str) -> bool{
    return Regex::new(r"[ \t]+").unwrap().is_match(&txt)
}

fn main(){
    let font_pt = 20;

    /*let font_name = "Amstelvar";
    let font_style = "Italic";*/

    let font_name = "Noto Sans CJK TC";
    let font_style = "Light";

    const PDF_WIDTH_IN_PX : f64 = 595.0;
    const PDF_HEIGHT_IN_PX : f64 = 842.0;
    let pdf_path = "/tmp/a.pdf";

    let mut regex_pattern1 = r"([^\s\p{Bopomofo}\p{Han}\p{Hangul}\p{Hiragana}\p{Katakana}。，、；：「」『』（）？！─……《》〈〉．～～゠‥｛｝［］〔〕〘〙〈〉《》【】〖〗※〳〵〴〲〱〽〃]{1,}|".to_string();
    let regex_pattern2 = r"[　\p{Bopomofo}\p{Han}\p{Hangul}\p{Hiragana}\p{Katakana}。，、；：「」『』（）？！─……《》〈〉．～～゠‥｛｝［］〔〕〘〙〈〉《》【】〖〗※〳〵〴〲〱〽〃]|──|〴〵|〳〵|[ \t]+)";
    regex_pattern1.push_str(&regex_pattern2);
    let regex_pattern = Regex::new(&regex_pattern1).unwrap();
    //let input_text = "我kā lí講這件——代誌彼は아버지 감사합니다といいます。It's true. happier. Ta̍k-ke. ٱلسَّلَامُ عَلَيْكُمْ שָׁלוֹם עֲלֵיכֶם";
    let input_text = "And why all this?d’aoís Certainly not because I believe that the land or the region has anything to do with it, for in any place and in any climate subjection is bitter and to be free is pleasant; but merely because I am of the opinion that one should pity those who, at birth, arrive with the yoke upon their necks. We should exonerate and forgive them, since they have not seen even the shadow of liberty, and, being quite unaware of it, cannot perceive the evil endured through their own slavery. If there were actually a country like that of the Cimmerians mentioned by Homer,";

    // 翻譯：在主後1602年，戰爭爆發於兩個以之間——以．歐尼爾和以．如瓦．歐唐納，在金特塞里附近，那時愛爾蘭人民在戰場激烈的耗了九年，對抗他們的敵人，為了……

    let mut input_text_vec = vec!();

    let block = Div{x:100.0,y:100.0,width:450.0, height: 250.0, lineskip: 30.0, language: "en", direction: harfbuzz_rs::Direction::Ltr, color: "#198964".to_string()};


    let mut text : String;
    for cap in regex_pattern.captures_iter(input_text){
        let text = cap[0].to_string().clone();

        input_text_vec.push(text);
    }


    let mut font_struct1 =  FontStruct{size:font_pt, name:font_name, style:font_style, variations : &[Variation::new(b"wght", 200.0),
         Variation::new(b"wdth", 20.0)], features : &[]};

    let box_coord_vec : Vec<Option<BoxCoodInfo>> = input_text_vec.into_iter().map(|x| get_box_cood_info(&x, font_struct1.name, font_struct1.style, font_struct1.size, block.language, harfbuzz_rs::Direction::Ltr, &[], &[])).collect();




    let surface = cairo::PdfSurface::new(PDF_WIDTH_IN_PX, PDF_HEIGHT_IN_PX, pdf_path).expect("Couldn’t create surface"); // A4 size

    let cxt = cairo::Context::new(&surface).expect("running error");




        cxt.set_source_rgba(0.8, 1.0, 1.0, 0.5); // 設定顏色
        cxt.paint().ok();// 設定背景顏色

        cxt.set_source_rgba(0.0, 0.0, 1.0, 1.0); // 設定顏色


        let font_struct2 =  FontStruct{size:30, name:"Noto Sans CJK TC", style:"Bold", variations : &[], features : &[]};

        let font_struct3 =  FontStruct{size:30, name:"Noto Nastaliq Urdu", style:"Bold", variations : &[], features : &[]};

        //layout_text("Tá grá agam duit", font_struct1,  100.0, 100.0,"#198964",harfbuzz_rs::Direction::Ltr, &cxt);

        //layout_text("انا احبك ", &font_struct3,  100.0, 300.0,"#198964",harfbuzz_rs::Direction::Rtl, &cxt);
        // println!("{:?}", result);

        greedy_typesetting(box_coord_vec, block, font_struct1, &cxt, false);
}
