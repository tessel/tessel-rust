var fs = require('fs');
var net = require('net');
var socketAddr = '/tmp/domain.sock';

var server = net.createServer(function(c) { //'connection' listener
    console.log('server connected');
    c.on('end', function() {
        // console.log('server disconnected');
    });
    c.on('data', function(d) {
      console.log("got data!", d);
      // c.end(d);
    });
});

server.on('error', function (e) {
    if (e.code == 'EADDRINUSE') {
        var clientSocket = new net.Socket();
        clientSocket.on('error', function(e) { // handle error trying to talk to server
            if (e.code == 'ECONNREFUSED') {  // No other server listening
                fs.unlinkSync(socketAddr);
                server.listen(socketAddr, function() { //'listening' listener
                    console.log('server recovered');
                });
            }
        });
        clientSocket.connect({path: socketAddr}, function() {
            console.log('Server running, giving up...');
            process.exit();
        });
    }
});

server.listen(socketAddr, () => console.log('server bound'));
