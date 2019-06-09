extends CanvasLayer

var address
var username

# Called when the node enters the scene tree for the first time.
func _ready():
	get_node("Control/Button").connect("pressed", self, "on_connect")
	var Laminar = load("res://laminar_client.gdns")
	var laminar = Laminar.new()
	laminar.new("127.0.0.1:12345")
	laminar.send("help")
	var got_packet: PoolByteArray = laminar.get_packet()
	print("Got packet: [", got_packet.get_string_from_utf8(),"]")
	
func on_connect():
	address = get_node("Control/ServerAddrBox").text
	username = get_node("Control/UsernameBox").text
	var global = get_node("/root/Global")
	global.username = username
	global.address = address
	#get_tree().change_scene("res://Game.tscn")

	