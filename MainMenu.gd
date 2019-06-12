extends CanvasLayer

var address
var username
var laminar 
# Called when the node enters the scene tree for the first time.
func _ready():
	get_node("Control/Button").connect("pressed", self, "on_connect")
	var Laminar = load("res://laminar_client.gdns")
	laminar = Laminar.new()
	#laminar.set_recv_callback(self as Node)
	address = get_node("Control/ServerAddrBox").text.strip_edges()
	laminar.new_connection(address)
	laminar.start_receiving(self as Node)
	
func on_connect():
	address = get_node("Control/ServerAddrBox").text.strip_edges()
	username = get_node("Control/UsernameBox").text
	var global = get_node("/root/Global")
	global.username = username
	global.address = address
	var pack = "reg:" + username + "," + username.to_lower() + String(OS.get_unix_time())
	laminar.send(pack)
	var pack1 = "note"
	laminar.send(pack1)
	#var got_packet: PoolByteArray = laminar.get_packet()
	#print("Got packet: [", got_packet.get_string_from_utf8(),"]")
	#get_tree().change_scene("res://Game.tscn")

func on_network_received(data):
	print("MainMenu got data: ", data)
	var packet = data
	#packet = data.split(":")
	if (packet == "reg:success"):
		get_tree().change_scene("res://Game.tscn")