#![allow(non_snake_case)]
extern crate crossbeam;
use std::net::{TcpListener, TcpStream};
use std::io::{BufReader,BufRead,Write};
use std::collections::HashMap;

// 独自例外クラス
#[derive(Debug)]
enum HTTPServerError{
    IO(std::io::Error)
}

impl From<std::io::Error> for HTTPServerError {
    fn from(err: std::io::Error) -> HTTPServerError {
        HTTPServerError::IO(err)
    }
}

// サーバ
struct HTTPServer{
    listener:TcpListener,
    httpContentHandler:fn(stream:&mut TcpStream,httpRequestHeaders:HashMap<String,String>)->Result<(),HTTPServerError>,
}

impl HTTPServer{
    fn new(binderAddress:&'static str)->Result<HTTPServer,HTTPServerError>{
        match TcpListener::bind(binderAddress){
            Ok(r)=>Ok(
                HTTPServer{
                        listener:r,
                        httpContentHandler:HTTPServer::httpContentHandler
                    }
                ),
            Err(e)=>Err(std::convert::From::from(e)),
        }
    }

    fn httpContentHandler(_:&mut TcpStream,_:HashMap<String,String>)->Result<(),HTTPServerError>{
        Err(std::convert::From::from(std::io::Error::new(std::io::ErrorKind::Other,"Method not implemented.")))
    }

    fn requestHandler(&self,stream:&mut TcpStream)->Result<(),HTTPServerError>{
        let mut reader = BufReader::new(stream);
        let mut recvLine:String = String::new();
        let mut requestHeader:HashMap<String,String>=HashMap::new();
        let _ = match reader.read_line(&mut recvLine){
                Ok(size)=>{
                    if size==0{
                        return Err(std::convert::From::from(std::io::Error::new(std::io::ErrorKind::BrokenPipe,"Broken pipe.")))
                    }
                }
                Err(_)=>return Err(std::convert::From::from(std::io::Error::new(std::io::ErrorKind::ConnectionAborted,"Connection aborted.")))
        };
        requestHeader.insert("requestLine".to_string(),recvLine);
        loop {
            recvLine=String::new();
            let _ = match reader.read_line(&mut recvLine){
                Ok(size)=>{
                    if size==0{
                        return Err(std::convert::From::from(std::io::Error::new(std::io::ErrorKind::BrokenPipe,"Broken pipe.")))
                    }
                },
                Err(_)=>return Err(std::convert::From::from(std::io::Error::new(std::io::ErrorKind::ConnectionAborted,"Connection aborted."))),
            };
            
            if recvLine!="\r\n"{
                let vOff = recvLine.find(':').unwrap_or(recvLine.len())+1;
                let k:String = recvLine.drain(..vOff).collect();
                requestHeader.insert(k.to_lowercase().replace(":",""),recvLine.replace("\r\n","").replace(" ",""));
            }else{
                break;
            }
        }
        let mut stream:&mut TcpStream = reader.get_mut();
        (self.httpContentHandler)(&mut stream,requestHeader)
    }

    fn setRequestHandler(&mut self,httpContentHandler:fn(stream:&mut TcpStream,httpRequestHeaders:HashMap<String,String>)->Result<(),HTTPServerError>){
        self.httpContentHandler=httpContentHandler;
    }

    fn listenServer(&self){
        crossbeam::scope(|scope| {
            for stream in self.listener.incoming(){
                let mut stream = match stream{
                    Ok(s)=>s,
                    Err(_)=>continue
                };
                let h = scope.spawn(move|| 
                match self.requestHandler(&mut stream){
                    Ok(_)=>{},
                    Err(e)=>{
                        println!("{:?}",e);
                    }
                });
                println!("next stream.");
            }
        });
    }
}

fn getHttpHeader(reqHeader:HashMap<String,String>,key:&str)->String{
    match reqHeader.get(&key.to_string()){
        Some(header) => header.to_string(),
        None=>{"".to_string()}
    }
}

fn requestHandler(stream:&mut TcpStream,reqHeader:HashMap<String,String>)->Result<(),HTTPServerError>{
    // リクエストラインを表示
    println!("request Line: {}",getHttpHeader(reqHeader,"requestLine"));
    // それっぽいレスポンスを投げる（失敗したらそのまま呼び元に例外投げる）
    try!(stream.write("HTTP/1.1 200 OK\r\n".as_bytes()));
    try!(stream.write("Date: Sun, 11 Jan 2004 16:06:23 GMT\r\n".as_bytes()));
    try!(stream.write("Connection: close\r\n".as_bytes()));
    try!(stream.write("Content-Type: text/plain\r\n".as_bytes()));
    try!(stream.write("\r\n".as_bytes()));
    try!(stream.write("hogehoge".as_bytes()));
    std::thread::sleep(std::time::Duration::from_millis(10000));
    Ok(())
}


fn main() {
    let mut svr:HTTPServer = match HTTPServer::new("127.0.0.1:19999"){
        Ok(r)=>r,
        Err(_)=>{
            println!("bind fail.");
            return;
        }
    };
    svr.setRequestHandler(requestHandler);
    svr.listenServer();
}
