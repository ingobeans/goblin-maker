from flask import Flask
from flask import request
from flask_cors import CORS, cross_origin
import os

app = Flask(__name__)
cors = CORS(app)

file_path = os.path.realpath(__file__)
levels_path = os.path.join(os.path.abspath(os.path.join(file_path, os.pardir)),"levels")

if not os.path.isdir(levels_path):
    os.mkdir(levels_path)

@app.route("/")
@cross_origin()
def home():
    return "<p>hello! this is the backend server for my videogame called 'Goblin Maker'</p>\n<p>you can try it <a href='https://github.com/ingobeans/goblin-maker'>here</a></p>"
@app.route("/list")
@cross_origin()
def list():
    return ",".join(os.listdir(levels_path))
@app.route("/get/<id>")
@cross_origin()
def get(id: str):
    if not all([c.isalnum() and c.isascii() or c == ' ' or c == '-' for c in id]):
        return "error:invalid id"
    if not "-" in id:
        return "error:missing author information"
    if len(id.split("-")[0]) > 20:
        return "error:too long name"
    if len(id.split("-")[1]) > 25:
        return "error:too long author name"
    path = os.path.join(levels_path,id)
    if not os.path.isfile(path):
        return "error:level doesn't exist"
    with open(path,"r") as f:
        return f.read()
@app.route("/upload/<id>")
@cross_origin()
def upload(id: str):
    if not all([c.isalnum() and c.isascii() or c == ' ' or c == '-' for c in id]):
        return "error:invalid id"
    if not "-" in id:
        return "error:missing author information"
    if len(id.split("-")[0]) > 20:
        return "error:too long name"
    if len(id.split("-")[1]) > 25:
        return "error:too long author name"
    data = request.headers.get("data")
    if data is None:
        return "error:missing data!"

    path = os.path.join(levels_path,id)
    if os.path.isfile(path):
        return "error:level name taken!"
    with open(path,"w") as f:
        f.write(data)
    return "ok"
app.run(port=5462)