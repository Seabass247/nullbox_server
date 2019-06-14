extends CanvasLayer

var address
var username 
var laminar
var global
# Called when the node enters the scene tree for the first time.
func _ready():
	get_node("Control/Button").connect("pressed", self, "on_connect")
	laminar = get_node("/root/Global").laminar
	address = get_node("Control/ServerAddrBox").text.strip_edges()
	
func on_connect():
	address = get_node("Control/ServerAddrBox").text.strip_edges()
	username = get_node("Control/UsernameBox").text
	global = get_node("/root/Global")
	global.init_client(username, address)
	var pack = "regplr:" + username
	laminar.send(pack)
	#var got_packet: PoolByteArray = laminar.get_packet()
	#print("Got packet: [", got_packet.get_string_from_utf8(),"]")
	#get_tree().change_scene("res://Game.tscn")

func on_network_received(data):
	print("MainMenu got data: ", data)
	var packet = data
	packet = data.split(";")
	if (packet[0] == "reg:success"):
		print("MainMenu: got network id " + packet[1] + " from the server")
		global.network_id = int(packet[1])
		get_tree().change_scene("res://Game.tscn")