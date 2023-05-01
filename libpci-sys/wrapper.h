#include <pci/pci.h>

extern void pci_lookup_class_helper(struct pci_access* pacc, char* class_str, size_t size_of, struct pci_dev* dev);
extern void pci_lookup_vendor_helper(struct pci_access* pacc, char* vendor, size_t size_of, struct pci_dev* dev);
extern void pci_lookup_device_helper(struct pci_access* pacc, char* device, size_t size_of, struct pci_dev* dev);
