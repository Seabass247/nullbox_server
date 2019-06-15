extends Node

var laminar

var username
var network_id

func _ready():
    #/var Laminar = load("res://laminar_client.gdns")
    #laminar = Laminar.new()
    laminar = get_node("/root/Laminar")

func init_client(username, address):
    laminar.new_connection(address)
    laminar.start_receiving(self as Node)
