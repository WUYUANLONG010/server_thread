
use pnet::datalink::{self, DataLinkReceiver, DataLinkSender, EtherType};
use pnet::datalink::NetworkInterface;
use pnet::packet::ip::IpNextHeaderProtocols::{self, LocalNetwork};
use pnet::packet::{self, Packet};
use pnet::datalink::Channel::Ethernet;
use pnet::packet::ethernet::{EtherTypes, EthernetPacket, MutableEthernetPacket};
use pnet::packet::ipv4::Ipv4Packet;
use rand::Rng;
use core::panic;
use std::f32::consts::E;
use std::sync::{Arc, Mutex};
use std::process::Output;
use std::slice::Chunks;
use std::thread::sleep;
use std::time::{Duration, Instant};
use std::{array, option, vec};
use std::thread;
// use time::{Duration, Timer};
const MAC_ADDR_LOCAL:datalink::MacAddr=datalink::MacAddr(0xc8,0x6b,0xbc,0x80,0x06,0x6a);
// const MAC_ADDR_SERVER:datalink::MacAddr=datalink::MacAddr(0x08,0x8e,0x90,0xf6,0x67,0x9c);
const MAC_ADDR_SERVER:datalink::MacAddr=datalink::MacAddr(0x6c,0x24,0x08,0xbe,0xc4,0x46);

struct PLC{
    is_main:bool,//主机：1，备机：0
    dig_out:[bool;1000],//数字输出变量
    dig_in:[bool;1000],//数字输入变量
    datalink_channel:NetworkInterface,//连接端口
    mac_addr:datalink::MacAddr,//自身mac地址
    is_decide:bool,
}
impl PLC {
    pub fn new(is_main:bool,
        dig_out:[bool;1000],
        dig_in:[bool;1000],
        mac_addr:datalink::MacAddr,
        is_decide:bool
    )->PLC{
        //初始化通信接口
        let interfaces: Vec<NetworkInterface> = datalink::interfaces();
        let interface=interfaces
        .into_iter()
        .filter(|iface:&NetworkInterface|iface.mac==Some(mac_addr))
        .next()
        .expect("Error get interface");
        PLC{
            is_main:is_main,
            dig_out:dig_out,
            dig_in:dig_in,
            datalink_channel: interface,
            mac_addr:mac_addr,
            is_decide,
        }
    }
    //初始化决定主备？？
    pub fn send_data_decide(&self){
        //开启数据传输通道
        let (mut tx,mut rx)= match datalink::channel(&self.datalink_channel, Default::default()){
            Ok(Ethernet(tx,rx)) =>(tx,rx),
            Ok(_)               =>panic!("Other"),
            Err(e)       =>panic!("error:{}",e),
        };
        loop{
            match  rx.next(){
                Ok(packet)=>{
                    let packet =EthernetPacket::new(packet).unwrap();
                    // client_handle_receive_packet_mac(&packet);
                }
                Err(e)=>{
                    println!("Some Error {}",e)
                }
            }
        }
        //
    }
    pub fn send_self_data_tx(&self,sign:&[u8;3],mac_other:datalink::MacAddr,tx:Arc<Mutex<Box<(dyn DataLinkSender + 'static)>>>){
        //向目的地址发送一次自身的数据：测试用
        let (dig_in,dig_out)=self.get_status();
        // println!("dig_in:{},dig_out:{}",dig_in.len(),dig_out.len());
        // let payload=self.dig_out;
        //新建payload作为传输的数据
        let start_time = Instant::now();
        let mut payload=Vec::new();
        payload.extend(sign);
        payload.extend(dig_out);
        payload.extend(dig_in);
        //缓冲区:数据长度+14字节头
        // let leng_buffer=payload.len()+3;
        // let mut buffer=Vec::with_capacity(leng_buffer);
        // println!("bufferlenth:{},payload length:{}",leng_buffer,payload.len());
        // let mut buffer: [u8; 1000] = [0; 1000];
        let mut buffer = vec![0u8; payload.len()+14];
        // println!("bufferlenth:{},payload length:{}",buffer.len(),payload.len());
        let mut ethernet_packet = MutableEthernetPacket::new(&mut buffer).unwrap();
        ethernet_packet.set_destination(mac_other); // 设置目标 MAC 地址
        ethernet_packet.set_source(self.mac_addr); // 设置源 MAC 地址
        ethernet_packet.set_ethertype(EtherTypes::Ipv4); // 设置帧类型为 IPv4
        ethernet_packet.set_payload(&payload);
        // println!("payload length:{:?},packet length:{:?}",payload.len(),ethernet_packet.packet().len());
        let mid_time = Instant::now();
        tx.lock().unwrap().send_to(ethernet_packet.packet(), Some(self.datalink_channel.clone()));
        let end_time = Instant::now();
        let pay_time = mid_time - start_time;
        let send_time = end_time - mid_time;
        // println!("pay time: {} milliseconds,send_time{}milliseconds", pay_time.as_micros(),send_time.as_micros());

    }
    pub fn send_self_data(&self,sign:&[u8;3],mac_other:datalink::MacAddr){
        //向目的地址发送一次自身的数据：测试用
        let (mut tx,mut rx)= match datalink::channel(&self.datalink_channel, Default::default()){
            Ok(Ethernet(tx,rx)) =>(tx,rx),
            Ok(_)               =>panic!("Other"),
            Err(e)       =>panic!("error:{}",e),
        };

        
        let (dig_in,dig_out)=self.get_status();
        // println!("dig_in:{},dig_out:{}",dig_in.len(),dig_out.len());
        // let payload=self.dig_out;
        //新建payload作为传输的数据
        let mut payload=Vec::new();
        payload.extend(sign);
        payload.extend(dig_out);
        payload.extend(dig_in);
        //缓冲区:数据长度+14字节头
        // let leng_buffer=payload.len()+3;
        // let mut buffer=Vec::with_capacity(leng_buffer);
        // println!("bufferlenth:{},payload length:{}",leng_buffer,payload.len());
        // let mut buffer: [u8; 1000] = [0; 1000];
        let mut buffer = vec![0u8; payload.len()+14];
        // println!("bufferlenth:{},payload length:{}",buffer.len(),payload.len());
        let mut ethernet_packet = MutableEthernetPacket::new(&mut buffer).unwrap();
        ethernet_packet.set_destination(mac_other); // 设置目标 MAC 地址
        ethernet_packet.set_source(self.mac_addr); // 设置源 MAC 地址
        ethernet_packet.set_ethertype(EtherTypes::Ipv4); // 设置帧类型为 IPv4
        ethernet_packet.set_payload(&payload);
        // println!("payload length:{:?},packet length:{:?}",payload.len(),ethernet_packet.packet().len());

        tx.send_to(ethernet_packet.packet(), Some(self.datalink_channel.clone()));

    }
    //随机生成数据,模拟读取IO数据，作为主机更新自身的变量
    pub fn change_data(&mut self){
        let mut rng = rand::thread_rng();
        let mut virtual_dig_out: [bool; 1000] = [false; 1000];
        for i in 0..virtual_dig_out.len() {
            virtual_dig_out[i] = rng.gen();
        }
        // println!("{:?}", &virtual_dig_out[..10]);
        
        let mut virtual_dig_in: [bool; 1000] = [false; 1000];
        for i in 0..virtual_dig_in.len() {
            virtual_dig_in[i] = rng.gen();
        }
        self.dig_in=virtual_dig_in;
        self.dig_out=virtual_dig_out;

    }
    pub fn decoder_packet_to_renew_ourself(&self,s:&[u8]){
        println!("receive data:{:?}",s);
    }
    //获取自身状态,以u8输出
    pub fn get_status(&self)->(Vec<u8>,Vec<u8>){
        let output_digin=self.dig_in;
        let output_digout=self.dig_out;
        // println!("{:?}",&self.dig_in[..10]);
        Self::change_bool_to_u8(output_digin,output_digout)
    }
    pub fn main_machine_handle_tx(&mut self,tx:Arc<Mutex<Box<dyn DataLinkSender>>>){
        //发送线程
        // let tx_clone=Arc::clone(&tx);
        loop {
            let tx_clone = Arc::clone(&tx);
            let sign: [u8; 3]=[255,255,255];
            // self.change_data();
            self.send_self_data_tx(&sign,MAC_ADDR_LOCAL,tx_clone);

        }
    }
    pub fn main_machine_handle_rx(&mut self,mut rx:Arc<Mutex<Box<dyn DataLinkReceiver>>>){
        //接收进程
        loop{
            match rx.lock().unwrap().next(){
                Ok(s)=>{
                    // println!("receive:data{:?}",s)
                    self.decoder_packet_to_renew_ourself(s);
                }
                Err(_)=>{
                    println!("error receive")
                }
            }
        }
    }
    pub fn main_machine_handle(&mut self){

        //进入主机处理循环
        loop {
            //主机未发现备机,按照主机运行
            if self.is_decide==false&&self.is_main==true{
                let sign: [u8; 3]=[255,255,255];
                self.change_data();
                println!("change_complete{:?}",&self.dig_out[..10]);
                self.send_self_data(&sign,MAC_ADDR_LOCAL);
                // self.send_self_data(sign, mac_other)
                //读取本机数据并处理
                // self.change_data();
                // println!("dig_in{:?}",self.dig_in);
                // println!("dig_out{:?}",self.dig_in);
                // panic!();
            }else {
            //主机降备机，退出主机处理循环
            break;
            }
        }
    }
    
    //得到传过来的数据
    fn renew_1ms(&self){

    }
    //主动主备切换
    fn change_main(&self){
        
    }
    //发送检测存活数据
    fn send_1ms(&self){

    }
    //将bool数组转换成u8数组
    pub fn change_bool_to_u8(dig_in:[bool;1000],dig_out:[bool;1000])->(Vec<u8>,Vec<u8>){


        let mut dig_in_array=Vec::new();
        for chunk in dig_in.chunks(8){
            let mut byte_value: u8 = 0;
            for (k,&bit) in chunk.iter().enumerate(){
                
                if bit{
                    byte_value|= 1 << (7-k);
                }
                ;
            }
            dig_in_array.push(byte_value)
        }
        let mut dig_out_array=Vec::new();
        // println!("{:?}",dig_out_array.len());
        // panic!();
        // let dig_out=[true;1000];
        //顺序截取每8位转换成u8
        for chunk in dig_out.chunks(8){
            let mut byte_value: u8 = 0;
            for (k,&bit) in chunk.iter().enumerate(){
                // println!("chunk:{:?}",chunk);
                // println!("k:{:?}, bit{:?}",&k,bit);
                if bit{
                    byte_value |= 1 << (7-k);
                }
            }
            // println!("byte value:{:?}",byte_value);
            dig_out_array.push(byte_value);
            // println!("{:?}",dig_out_array);
            // panic!();
            // println!("byte value:{:?}",byte_value);
            
        }
        // println!("transfrom complete{:?}",dig_out_array);
        // panic!();
        (dig_in_array,dig_out_array)
    }
    //输入为可变长度的Vec时进行转换
    //长度转换了，值没有转换
    pub fn single_change_bool_to_u8(input:Vec<bool>)->Vec<u8>{
        let mut dig_in_array: Vec<u8>=Vec::new();
        for chunk in input.chunks(8){
            let mut byte_value: u8 = 0;
            for (k,&bit) in chunk.iter().enumerate(){
                if bit{
                    byte_value |= 1 << (7-k);
                }
                ;
            }
            dig_in_array.push(byte_value)
        }
        dig_in_array
    }
}
fn main() {
    // println!("Hello, world!");
    let mut rng = rand::thread_rng();
    let mut virtual_dig_out: [bool; 1000] = [false; 1000];
    for i in 0..virtual_dig_out.len() {
        virtual_dig_out[i] = rng.gen();
    }
    // println!("{:?}", &virtual_dig_out[..10]);
    
    let mut virtual_dig_in: [bool; 1000] = [false; 1000];
    for i in 0..virtual_dig_in.len() {
        virtual_dig_in[i] = rng.gen();
    }
    // println!("{:?}", &virtual_dig_in[..10]);

    // let mut plc=PLC::new(true, virtual_dig_out, virtual_dig_in, MAC_ADDR_SERVER,false);
    let plc = Arc::new(Mutex::new(
        PLC::new(true, virtual_dig_out, virtual_dig_in, MAC_ADDR_SERVER,false)));
        //开启通信接口
    let (mut tx,rx)=
     match datalink::channel(&plc.lock().unwrap().datalink_channel, Default::default()){
        Ok(Ethernet(tx,rx)) =>(tx,rx),
        Ok(_)               =>panic!("Other"),
        Err(e)       =>panic!("error:{}",e),
    };
    //开启发送进程
    let shared_rx=Arc::new(Mutex::new(rx));
    let shared_tx=Arc::new(Mutex::new(tx));
    let shared_plc=Arc::clone(&plc);
    let shared_plc_2=Arc::clone(&plc);
    let handle_send=thread::spawn(move||{
        shared_plc.clone().lock().unwrap().main_machine_handle_tx(Arc::clone(&shared_tx));
    });
    let handle_receive=thread::spawn(move||{
        shared_plc_2.clone().lock().unwrap().main_machine_handle_rx(Arc::clone(&shared_rx));
    });

    // handle_receive_2.join().unwrap();
    handle_send.join().unwrap();
    handle_receive.join().unwrap();


}
