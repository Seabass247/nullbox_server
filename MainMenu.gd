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
	var msg = "gaga"
	var arr = [5, "six"]
	laminar.test("gaga" as String, arr)
	
func on_connect():
	address = get_node("Control/ServerAddrBox").text.strip_edges()
	username = get_node("Control/UsernameBox").text
	global = get_node("/root/Global")
	laminar.init_client(address as String, self as Node)
	var pack: Array = ["register:", username]
	laminar.send_vars("/root/Game:player_connected" as String, pack)
	#var got_packet: PoolByteArray = laminar.get_packet()
	#print("Got packet: [", got_packet.get_string_from_utf8(),"]")
	#get_tree().change_scene("res://Game.tscn")

func _on_net_server_response(data):
	
	print("Server connect response: ", data)
	
	if (false):
		#print("MainMenu got net id: ", id, " and status: ", status)
		#global.network_id = int(data[1][0])
		get_tree().change_scene("res://Game.tscn")