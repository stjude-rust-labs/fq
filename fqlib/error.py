class FastQValidationError(Exception):
    
    def __init__(self, description, readname, filename):
        self.description = description
        self.readname = readname
        self.filename = filename
    
    def __str__(self):
        return "Read '{readname}' failed validation in file {filename} for the following " \
               "reason: {description}".format(**self.__dict__)
