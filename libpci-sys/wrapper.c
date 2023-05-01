#include "wrapper.h"

void pci_lookup_class_helper(struct pci_access* pacc, char* class_str, size_t size_of, struct pci_dev* dev) {
    pci_lookup_name(pacc, class_str, size_of, PCI_LOOKUP_CLASS, dev->device_class);
}

void pci_lookup_vendor_helper(struct pci_access* pacc, char* vendor, size_t size_of, struct pci_dev* dev) {
    pci_lookup_name(pacc, vendor, size_of, PCI_LOOKUP_VENDOR, dev->vendor_id, dev->device_id);
}

void pci_lookup_device_helper(struct pci_access* pacc, char* device, size_t size_of, struct pci_dev* dev) {
    pci_lookup_name(pacc, device, size_of, PCI_LOOKUP_DEVICE, dev->vendor_id, dev->device_id);
}
