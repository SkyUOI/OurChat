import flask
from flask import request

app = flask.Flask(__name__)

@app.route("/upload", methods=["POST"])
def upload():
    request.headers["Key"],request.get_data()
    return "ok"

if __name__ == "__main__":
    app.run(host="0.0.0.0", port=7778)