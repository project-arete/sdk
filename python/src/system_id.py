import os
import uuid

LINUX_MODEL_FILENAME = '/sys/firmware/devicetree/base/model'
LINUX_SERIAL_NUMBER_FILENAME = '/sys/firmware/devicetree/base/serial-number'

def get_system_id():
    if os.path.isfile(LINUX_MODEL_FILENAME) and os.path.isfile(LINUX_SERIAL_NUMBER_FILENAME):
        return get_system_id_linux()
    else:
        raise Exception('Unable to detect System ID on this platform')

def get_system_id_linux():
    with open(LINUX_MODEL_FILENAME, 'r') as file:
        model = file.read()
    with open(LINUX_SERIAL_NUMBER_FILENAME, 'r') as file:
        serial_number = file.read()
    model_plus_serial_number = f"{model}:{serial_number}"

    id = uuid.uuid5(uuid.NAMESPACE_OID, model_plus_serial_number)
    return id
