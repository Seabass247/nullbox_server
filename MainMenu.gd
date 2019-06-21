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
	laminar.init_client(address as String)
	laminar.set_root("/root/MainMenu")
	var pack: Array = ["register:", username]
	laminar.send("/root/Game:player_connected" as String, pack as Array)
	
func _on_net_server_response(data):
	print("Server connect response: ", data)
	if (data[0] == "success"):
		get_tree().change_scene("res://Game.tscn")
		