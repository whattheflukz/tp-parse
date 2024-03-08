#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use clap::{Parser, ValueEnum};
use std::error::Error;
use std::io;
use csv;
use serde;

#[derive(Debug, Parser)]
#[command(author="Keith Sachs", version="0.0.1", about="a simple parser that takes in pick and place files, picks out required testpoints, and processes them for use on testheads", long_about = None)]
#[command(version = "0.1")]
struct Args {

    /// Name of the file being input
    #[arg(short, long)]
    input_file: String,
    
    /// Name of the file being created
    #[arg(short,long)]
    output_file: Option<String>,
    
    /// Include the descriptors
    #[arg(short,long)]
    name: Option<bool>,

    /// rotate the points 
    #[arg(short,long)]
    rotation: Option<Rotation>,

    //override include designator
    #[arg(short='c',long ="inclusions", value_delimiter=',')]
    inclusion: Option<Vec<String>>,
}

#[derive(Debug, serde::Deserialize, Clone, ValueEnum)]
enum Rotation {
    R90 = 90,
    R180 = 180,
    R270 = 270,
}

#[derive(Debug, serde::Deserialize, Clone, PartialEq)]
struct PTS {
    designator: String,
    comment: String,
    layer: String,
    footprint: String,
    x: f64,
    y: f64,
    rotation: f64,
    description: String,
}

fn read_csv(path: String, override_inclusions: Option<Vec<String>>) -> Result<Vec<PTS>, Box<dyn Error>> {
    let mut output = vec!();
    let mut reader = csv::Reader::from_path(path)?;
    let inclusions = match override_inclusions {
        Some(i) => {
            println!("{:?}", i);
            i
        },
        None => Vec::new(),
    };

    for result in reader.deserialize(){
        let record: PTS = result?;
        if (record.designator.contains("TP") && record.layer.to_lowercase().contains("bottom")) || (record.designator.contains("FD") && record.layer.to_lowercase().contains("bottom")) || record.designator.contains("J"){
            output.push(record);
        }
        else if !inclusions.is_empty() {
            for inclusion in &inclusions{
                if record.designator.contains(inclusion.as_str()){
                    output.push(record.clone());
                    println!("Including: {:?}", inclusion);
                }
            }
        }

    }
    Ok(output)
}

// gets the average center of the vec
fn get_mid_point(pt_vec: &Vec<PTS>) -> (f64, f64) {
    let l = pt_vec.len() as f64;
    let mut x: f64 = 0.0;
    let mut y: f64 = 0.0;
    for pt in pt_vec {
        x = x + pt.x;
        y = y + pt.y;
    }
    x = x/l;
    y = y/l;
    println!("mid point x: {:?} \nmid point y: {:?}", x, y);
    (x,y)
}

// rotates the vec by 90, 180, or 270 degrees
fn rotate_vec(deg: Rotation, pt_vec: &mut Vec<PTS>) {
    println!("Rotating vec by: {:?}", deg);
    let mut out: Vec<PTS> = vec!();
    match deg {
        Rotation::R90 => {
            for pt in &mut *pt_vec {
                let mut p = pt.clone();
                p.x = pt.y;
                p.y = -(pt.x);
                // println!("p.x: {:?}, p.y: {:?}", &p.x, &p.y);
                out.push(p);
            }
        },
        Rotation::R180 => {
            for pt in &mut *pt_vec {
                let mut p = pt.clone();
                p.x = -(pt.x);
                p.y = -(pt.y);
                // println!("p.x: {:?}, p.y: {:?}", &p.x, &p.y);
                out.push(p);
            }
        },
        Rotation::R270 => {
            for pt in &mut *pt_vec {
                let mut p = pt.clone();
                // p.x = pt.x;
                p.y = -(pt.y);
                // println!("p.x: {:?}, p.y: {:?}", &p.x, &p.y);
                out.push(p);
            }
        },
    }
    *pt_vec = out.clone();
}

// moves vec into quadrant 1 so the bottom left corner is 0,0
fn offset_vec(pt_vec: &mut Vec<PTS>){
    let (mid_x, mid_y) = get_mid_point(&pt_vec);
    let offset_x = 3.5 - mid_x;
    let offset_y = 3.0 - mid_y;

    println!("Offsetting vec by :({:?},{:?}) midpoint: ({:?},{:?})", offset_x, offset_y, mid_x, mid_y);
    for pt in pt_vec{
        pt.x = pt.x + offset_x;
        pt.y = pt.y + offset_y;
        // println!("pt.x: {:?}, pt.y: {:?}", pt.x, pt.y);
    }
}

// converts from mil to inches
fn scale_vec(pt_vec: &mut Vec<PTS>) {
    println!("Scaling vec");
    for pt in pt_vec {
        pt.x = pt.x / 1000.0;
        pt.y = pt.y / 1000.0;
        // println!("pt.x: {:?}, pt.y: {:?}", pt.x, pt.y);
    }
}

// adds the 6 screws plus the two alignment pins on the testhead. 
fn add_screws(pt_vec: &mut Vec<PTS>){
    pt_vec.push(
        PTS{
            designator: "E1".to_string(),
            comment: "alignment pin".to_string(),
            layer: "all".to_string(),
            footprint: "".to_string(),
            x: 0.5,
            y: 3.0,
            rotation: 0.0,
            description: "alignment pin".to_string(),
        }
    );   
    pt_vec.push(
        PTS{
            designator: "E2".to_string(),
            comment: "alignment pin".to_string(),
            layer: "all".to_string(),
            footprint: "N/A".to_string(),
            x: 6.5,
            y: 3.0,
            rotation: 0.0,
            description: "alignment pin".to_string(),
        }
    );
    pt_vec.push(
        PTS{
            designator: "D1".to_string(),
            comment: "alignment pin".to_string(),
            layer: "N/A".to_string(),
            footprint: "N/A".to_string(),
            x: 0.5,
            y: 0.25,
            rotation: 0.0,
            description: "alignment pin".to_string(),
        }
    );
    pt_vec.push(
        PTS{
            designator: "D2".to_string(),
            comment: "alignment pin".to_string(),
            layer: "N/A".to_string(),
            footprint: "N/A".to_string(),
            x: 0.5,
            y: 5.75,
            rotation: 0.0,
            description: "alignment pin".to_string(),
        }
    );
    pt_vec.push(
        PTS{
            designator: "D3".to_string(),
            comment: "alignment pin".to_string(),
            layer: "N/A".to_string(),
            footprint: "N/A".to_string(),
            x: 3.5,
            y: 0.25,
            rotation: 0.0,
            description: "alignment pin".to_string(),
        }
    );
    pt_vec.push(
        PTS{
            designator: "D4".to_string(),
            comment: "alignment pin".to_string(),
            layer: "N/A".to_string(),
            footprint: "N/A".to_string(),
            x: 3.5,
            y: 5.75,
            rotation: 0.0,
            description: "alignment pin".to_string(),
        }
    );
    pt_vec.push(
        PTS{
            designator: "D5".to_string(),
            comment: "alignment pin".to_string(),
            layer: "N/A".to_string(),
            footprint: "N/A".to_string(),
            x: 6.5,
            y: 0.25,
            rotation: 0.0,
            description: "alignment pin".to_string(),
        }
    );
    pt_vec.push(
        PTS{
            designator: "D6".to_string(),
            comment: "alignment pin".to_string(),
            layer: "N/A".to_string(),
            footprint: "N/A".to_string(),
            x: 6.5,
            y: 5.75,
            rotation: 0.0,
            description: "alignment pin".to_string(),
        }
    );

    pt_vec.push(
        PTS{
            designator: "SCREW1".to_string(),
            comment: "alignment pin".to_string(),
            layer: "N/A".to_string(),
            footprint: "N/A".to_string(),
            x: 2.77,
            y: 2.282,
            rotation: 0.0,
            description: "alignment pin".to_string(),
        } 
    );
    pt_vec.push(
        PTS{
            designator: "SCREW2".to_string(),
            comment: "alignment pin".to_string(),
            layer: "N/A".to_string(),
            footprint: "N/A".to_string(),
            x: 4.23,
            y: 2.282,
            rotation: 0.0,
            description: "alignment pin".to_string(),
        }
    );
    pt_vec.push(
        PTS{
            designator: "SCREW3".to_string(),
            comment: "alignment pin".to_string(),
            layer: "N/A".to_string(),
            footprint: "N/A".to_string(),
            x: 2.77,
            y: 3.718,
            rotation: 0.0,
            description: "alignment pin".to_string(),
        }
    );
    pt_vec.push(
        PTS{
            designator: "SCREW4".to_string(),
            comment: "alignment pin".to_string(),
            layer: "N/A".to_string(),
            footprint: "N/A".to_string(),
            x: 4.23,
            y: 3.718,
            rotation: 0.0,
            description: "alignment pin".to_string(),
        }
    );
}

fn size_holes(v: &mut Vec<PTS>){

    let mut v_cpy = v.clone();
    // let mut v_out: Result<Vec<PTS>, Box<dyn Error>> = Vec::new();
    for point in &mut *v {
        let mut size = 0.068;
        point.comment = String::from("100");
        // println!("{:?}, {:?}", point.x, point.y);
        for point_2 in &v_cpy {

            // println!("tp: {:?} tp_2 {:?} Point x: {:?} Point x2: {:?} Point y: {:?} Point y2: {:?} distance: {:?}", point.designator, point_2.designator,point.x / 1000.0, point_2.x / 1000.0, point.y / 1000.0, point_2.y / 1000.0, ((point.x - point_2.x).powf(2.0)+(point.y - point_2.y).powf(2.0)).sqrt().abs() / 1000.0 );

            let diff = ((point.x - point_2.x).powf(2.0)+(point.y - point_2.y).powf(2.0)).sqrt().abs() / 1000.0;
            if point.x == point_2.x && point.y == point_2.y {}

            else if diff < 0.085 {
                if diff < 0.068
                {
                    println!("WARNING, {:?} AND {:?} TOO CLOSE FOR 075 PROBES. DISTANCE: {:?}", point.designator, point_2.designator, diff);
                }
                point.comment = String::from("075");
                // println!("Size 075");
                size = 0.05511811;
                break;
            }
        }
    };

}

fn main() -> Result<(), Box<dyn Error>>{
    let args = Args::parse();

    let named = match args.name {
        Some(n) => n,
        _ => false,
    };

    let output_file = match args.output_file {
        Some(f) => f,
        _ => "output.csv".to_string(),
    };

    println!("selected file: {}", args.input_file);

    let mut wtr = csv::Writer::from_path(output_file)?;
    if named{
        let _ = wtr.write_record(&["x", "y", "radius", "designator"]);
    } else {
        let _ = wtr.write_record(&["x", "y", "radius"]);
    }
    let mut v = read_csv(args.input_file, args.inclusion).unwrap();

    size_holes(&mut v);

    // let mut v = match read_csv(args.input_file) {
    //     Some(expr) => expr,
    //     None => expr,
    // };
    
    scale_vec(&mut v);
    offset_vec(&mut v);

    match args.rotation {
        Some(r) => {
            rotate_vec(r, &mut v);
            offset_vec(&mut v);
        },
        _ => println!("no rotation"),
    };

    add_screws(&mut v);

    if named {
        for x in v{

            let mut size = 0.068;
            // println!("Comment: {:?}", x.comment);
            if x.comment.contains("075") { size = 0.05511811};
            if x.designator.chars().next().unwrap() == 'D' { size = 0.190 }
            if x.designator.chars().next().unwrap() == 'E' { size = 0.245 }
            if x.designator.chars().next().unwrap() == 'S' { size = 0.25 }
            if x.designator.chars().next().unwrap() == 'P' { size = 0.125 }

            let _ = wtr.write_record(&[format!("{:.4}", x.x.clone()), format!("{:.4}", x.y.clone()), size.to_string(), x.designator.clone().to_string()]);
            println!("designator: {:?}, x: {:?}, y: {:?}", x.designator, x.x, x.y);
        }
    } else {
        for x in v{
            // let mut size = 0.068;
            let mut size = 0.068;
            // println!("Comment: {:?}", x.comment);
            if x.comment.contains("075") { size = 0.05511811};
            if x.designator.chars().next().unwrap() == 'D' { size = 0.190 }
            if x.designator.chars().next().unwrap() == 'E' { size = 0.245 }
            let _ = wtr.write_record(&[format!("{:.4}", x.x.clone()), format!("{:.4}", x.y.clone()), size.to_string()]);
            println!("designator: {:?}, x: {:?}, y: {:?}", x.designator, format!("{:.4}", x.x), format!("{:.4}", x.y));
        }
    }
            
    wtr.flush()?;
    Ok(())
}
