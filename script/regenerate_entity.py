import os

os.system(
    "sea generate entity -u mysql://root:123456@localhost:3306/OurChat -o server/src/entities"
)
