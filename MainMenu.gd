extends CanvasLayer

var address
var port
var username
var socketUDP = PacketPeerUDP.new()

# Called when the node enters the scene tree for the first time.
func _ready():
	get_node("Control/Button").connect("pressed", self, "on_connect")
	socketUDP.listen(12346, "*", 65536)
	
func on_connect():
	address = get_node("Control/ServerAddrBox").text
	username = get_node("Control/UsernameBox").text
	var global = get_node("/root/Global")
	global.username = username
	global.address = address
	
	if socketUDP.is_listening():
		socketUDP.set_dest_address(address, 12351)
		var pac = "hi server!".to_utf8()
		print("pac: ", pac.get_string_from_utf8())
		socketUDP.put_packet(pac)
		socketUDP.put_packet(pac)
		socketUDP.put_packet(pac)
		socketUDP.put_packet(pac)
		socketUDP.put_packet(pac)
		socketUDP.put_packet(pac)
		socketUDP.put_packet(pac)
		socketUDP.put_packet(pac)
		socketUDP.put_packet(pac)
		socketUDP.put_packet(pac)
		socketUDP.put_packet(pac)
		socketUDP.put_packet(pac)
		socketUDP.put_packet(pac)
		socketUDP.put_packet(pac)
		print("sent packet")
	#get_tree().change_scene("res://Game.tscn")