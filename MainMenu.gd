extends CanvasLayer

var address
var username 
var laminar
# Called when the node enters the scene tree for the first time.
func _ready():
	get_node("Control/Button").connect("pressed", self, "on_connect")
	laminar = get_node("/root/Global").laminar
	address = get_node("Control/ServerAddrBox").text.strip_edges()
	
func on_connect():
	address = get_node("Control/ServerAddrBox").text.strip_edges()
	username = get_node("Control/UsernameBox").text
	var global = get_node("/root/Global")
	var network_id = username.to_lower() + String(OS.get_unix_time())
	global.init_client(username, address, network_id)
	var pack = "reg:" + username + "&" + network_id
	laminar.send(pack)
	var pack1 = "note:got it"
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