package discovery

import (
	"context"
	"fmt"
	"log"
	"net"
	"os"
	"os/signal"
	"syscall"

	"github.com/grandcat/zeroconf"
)

const (
	serviceName = "_pos-server._tcp"
	domain      = "local."
)

// Server represents a discovery server
type Server struct {
	server *zeroconf.Server
	port   int
}

// NewServer creates a new discovery server
func NewServer(port int) *Server {
	return &Server{
		port: port,
	}
}

// Start starts broadcasting the service
func (s *Server) Start() error {
	// Get hostname
	hostname, err := os.Hostname()
	if err != nil {
		return fmt.Errorf("failed to get hostname: %v", err)
	}

	// Create the service
	server, err := zeroconf.Register(
		hostname,                    // Instance name
		serviceName,                // Service type
		domain,                     // Domain
		s.port,                     // Port
		[]string{"version=1.0.0"},  // TXT records
		nil,                        // Interface binding (nil for all)
	)
	if err != nil {
		return fmt.Errorf("failed to register zeroconf service: %v", err)
	}

	s.server = server

	// Get local IP for logging
	ip, err := getLocalIP()
	if err != nil {
		log.Printf("Warning: Could not determine local IP: %v", err)
	} else {
		log.Printf("Broadcasting POS service on %s:%d", ip.String(), s.port)
	}

	// Handle graceful shutdown
	go func() {
		sigChan := make(chan os.Signal, 1)
		signal.Notify(sigChan, syscall.SIGINT, syscall.SIGTERM)
		<-sigChan
		s.Stop()
	}()

	return nil
}

// Stop stops broadcasting the service
func (s *Server) Stop() {
	if s.server != nil {
		log.Printf("Stopping Zeroconf server...")
		s.server.Shutdown()
		s.server = nil
		log.Printf("Zeroconf server stopped")
	}
}

// getLocalIP returns the non-loopback local IP of the host
func getLocalIP() (net.IP, error) {
	addrs, err := net.InterfaceAddrs()
	if err != nil {
		return nil, err
	}
	for _, address := range addrs {
		// Check the address type and if it is not a loopback
		if ipnet, ok := address.(*net.IPNet); ok && !ipnet.IP.IsLoopback() {
			if ipnet.IP.To4() != nil {
				return ipnet.IP, nil
			}
		}
	}
	return nil, fmt.Errorf("no local IP address found")
}

// Browse looks for POS servers on the local network
func Browse(ctx context.Context) ([]string, error) {
	resolver, err := zeroconf.NewResolver(nil)
	if err != nil {
		return nil, fmt.Errorf("failed to create resolver: %v", err)
	}

	entries := make(chan *zeroconf.ServiceEntry)
	servers := make([]string, 0)

	go func() {
		for entry := range entries {
			if len(entry.AddrIPv4) > 0 {
				servers = append(servers, fmt.Sprintf("http://%s:%d", entry.AddrIPv4[0], entry.Port))
			}
		}
	}()

	err = resolver.Browse(ctx, serviceName, domain, entries)
	if err != nil {
		return nil, fmt.Errorf("failed to browse: %v", err)
	}

	// Wait a bit for responses
	select {
	case <-ctx.Done():
		close(entries)
	}

	return servers, nil
}

// Global instance
var (
	globalServer *Server
	initialized  bool
)

// StartDiscoveryService initializes and starts the discovery service
func StartDiscoveryService() {
	if initialized {
		log.Printf("Discovery service already initialized, skipping...")
		return
	}
	
	log.Printf("Initializing discovery service...")
	globalServer = NewServer(8000)
	if err := globalServer.Start(); err != nil {
		log.Printf("Warning: Failed to start discovery service: %v", err)
		globalServer = nil
		return
	}
	
	initialized = true
	log.Printf("Discovery service started successfully")
	
	// Handle shutdown in a separate goroutine
	go func() {
		sigChan := make(chan os.Signal, 1)
		signal.Notify(sigChan, syscall.SIGINT, syscall.SIGTERM)
		<-sigChan
		
		log.Printf("Shutting down discovery service...")
		if globalServer != nil {
			globalServer.Stop()
			globalServer = nil
			initialized = false
		}
	}()
}
