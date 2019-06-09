extends CanvasLayer

var address
var username
var socketUDP = PacketPeerUDP.new()

# Called when the node enters the scene tree for the first time.
func _ready():
	get_node("Control/Button").connect("pressed", self, "on_connect")
	var Laminar = load("res://laminar_client.gdns")
	var laminar = Laminar.new()
	laminar.send("help")
	socketUDP.listen(12346, "*", 65536)
	
func on_connect():
	address = get_node("Control/ServerAddrBox").text
	username = get_node("Control/UsernameBox").text
	var global = get_node("/root/Global")
	global.username = username
	global.address = address
	#get_tree().change_scene("res://Game.tscn")