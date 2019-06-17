extends CanvasLayer

var address
var username 
var laminar
var global

# Called when the node enters the scene tree for the first time.
func _ready():
	get_node("Control/Button").connect("pressed", self, "on_connect")
	get_node("Control/Button2").connect("pressed", self, "on_button_test")
	laminar = get_node("/root/Global").laminar
	address = get_node("Control/ServerAddrBox").text.strip_edges()

func on_connect():
	address = get_node("Control/ServerAddrBox").text.strip_edges()
	username = get_node("Control/UsernameBox").text
	global = get_node("/root/Global")
	laminar.init_client(address as String, self as Node)
	var pack: Array = ["register:", username]
	laminar.send("/root/Game:player_connected" as String, pack)
	

func on_button_test():
	get_tree().change_scene("res://Game.tscn")
	
func _on_net_server_response(data):
	print("Server connect response: ", data)
	if (data[0] == "success"):
		#global.client_id = data[1]
		get_tree().change_scene("res://Game.tscn")
		laminar.sleep(5000)