fn main() {
	let _stream = rodio::OutputStreamBuilder::open_default_stream().unwrap();
	let sink = rodio::Sink::connect_new(&_stream.mixer());
	let file = std::fs::File::open("sounds/harp.ogg").unwrap();
	let source = rodio::Decoder::new(std::io::BufReader::new(file)).unwrap();
	sink.append(source);
	sink.sleep_until_end();
}