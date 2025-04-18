# DOCUMENTACIÓN DE LA API

Aqui puedes ver los datos necesarios para realizar cada fetch y los nombres que deben de llevar.

Todos los valores deben de ir en el body de la request/solicitud/fetch.

> [!IMPORTANT]
> Instalar la extensión [REST Client](https://marketplace.visualstudio.com/items/?itemName=humao.rest-client)

## Levantar el servidor

En la carpeta _api._

`cd atiaTTS/api/`

Con RUST previamente instalado ejecuta:

`cargo build`

`cargo run`

## RUST Client usage

**Enlace a la documentación de la [API](api.http)**

Arriba del método HTTP (POST, GET, PUT ...), si todo está instalado correctamente, aparecera al leyenda _Send Request_ o _Enviar Petición_ depende del idioma, ese botón realizará una petición al servidor.

1. El json debajo son los datos que el servidor necesita para realizar la petición

2. Una vez se procese la petición, en la pestaña nueva, estará la información que se enviá al frontEnd como respuesta. 
