extends CanvasLayer

var server_ip
var server_port
var username 
var laminar
var output
var global

# Called when the node enters the scene tree for the first time.
func _ready():
	get_node("Control/Button").connect("pressed", self, "on_connect")
	get_node("Control/ButtonLan").connect("pressed", self, "on_connect_lan")
	output = get_node("Control/OutputTextBox")
	laminar = get_node("/root/Global").laminar
	laminar.set_root("/root/MainMenu")
	
func on_connect():
	server_ip = get_node("Control/ServerIpBox").text.strip_edges()
	server_port = get_node("Control/ServerPortBox").text.strip_edges()
	username = get_node("Control/UsernameBox").text
	global = get_node("/root/Global")
	var address = server_ip + ":" + server_port
	laminar.init_client(address as String)
	laminar.set_root("/root/MainMenu")
	var pack: Array = ["register:", username]
	laminar.send("/root/Game:player_connected" as String, pack as Array)
	output.text = "Attempting connectiong to server"

func on_connect_lan():
	server_ip = "127.0.0.1"
	server_port = get_node("Control/ServerPortBox").text.strip_edges()
	username = get_node("Control/UsernameBox").text
	global = get_node("/root/Global")
	var address = server_ip + ":" + server_port
	laminar.init_client(address as String)
	laminar.set_root("/root/MainMenu")
	var pack: Array = ["register:", username]
	laminar.send("/root/Game:player_connected" as String, pack as Array)
	output.text = "Attempting connectiong to server"

func _on_net_server_response(data):
	print("Server connect response: ", data)
	if (data[0] == "success"):
		global.client_id = data[1]
		get_tree().change_scene("res://Game.tscn")
		
func _on_net_timed_out():
	output.text = "Connection to server timed out"