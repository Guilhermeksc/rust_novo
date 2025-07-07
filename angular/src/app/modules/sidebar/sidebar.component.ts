import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { NavigationService, MenuItem } from '../../services/navigation.service';

@Component({
    selector: 'app-sidebar',
    standalone: true,
    imports: [CommonModule],
    templateUrl: './sidebar.component.html',
    styleUrl: './sidebar.component.css'
})
export class SidebarComponent implements OnInit {
  menuItems: MenuItem[] = [];
  currentPage: string = '';
  isCollapsed: boolean = false;
  isMobileOpen: boolean = false;

  constructor(private navigationService: NavigationService) {}

  ngOnInit(): void {
    this.menuItems = this.navigationService.menuItems;
    
    this.navigationService.currentPage$.subscribe(page => {
      this.currentPage = page;
    });

    this.navigationService.sidebarCollapsed$.subscribe(collapsed => {
      this.isCollapsed = collapsed;
    });

    this.navigationService.sidebarOpen$.subscribe(open => {
      this.isMobileOpen = open;
    });
  }

  selectMenuItem(itemId: string): void {
    this.navigationService.setCurrentPage(itemId);
    
    // Close mobile sidebar when selecting an item
    if (this.isMobileOpen) {
      this.navigationService.setSidebarOpen(false);
    }
  }

  toggleSidebar(): void {
    this.navigationService.toggleSidebar();
  }
}
