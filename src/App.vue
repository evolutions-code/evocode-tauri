<template>
  <a-config-provider
    :theme="{
      algorithm: isDark ? theme.darkAlgorithm : theme.defaultAlgorithm,
      token: {
        colorPrimary: '#cccccc',
        colorInfo: '#cccccc',
        colorSuccess: '#34d399',
        colorWarning: '#fbbf24',
        colorError: '#f87171',
        colorBgBase: isDark ? '#000000' : '#f5f5f5',
        colorTextBase: isDark ? '#ffffff' : '#111111',
        borderRadius: 10,
        fontFamily: '-apple-system, BlinkMacSystemFont, Segoe UI, Roboto, Helvetica Neue, PingFang SC, Microsoft YaHei, sans-serif',
        controlHeight: 36,
        wireframe: false,
      },
      components: {
        Layout: {
          headerBg: 'transparent',
          siderBg: 'transparent',
          bodyBg: 'transparent',
        },
        Menu: {
          itemBg: 'transparent',
          subMenuItemBg: 'transparent',
          itemSelectedBg: 'rgba(255,255,255,0.10)',
          itemSelectedColor: '#f5f7fb',
          itemHoverBg: 'rgba(255,255,255,0.04)',
        },
        Card: {
          colorBgContainer: 'var(--bg-elev-2)',
        },
        Tabs: {
          itemColor: 'var(--text-3)',
          itemHoverColor: 'var(--text-1)',
          itemSelectedColor: 'var(--text-1)',
        },
        Button: {
          colorPrimaryHover: '#dddddd',
        },
        Input: {
          colorBgContainer: 'rgba(255,255,255,0.04)',
          colorBorder: 'rgba(255,255,255,0.12)',
          colorPrimaryHover: '#cccccc',
          colorPrimaryActive: '#cccccc',
        },
        Select: {
          colorBgContainer: 'rgba(255,255,255,0.04)',
          colorBorder: 'rgba(255,255,255,0.12)',
          colorPrimaryHover: '#cccccc',
          colorPrimaryActive: '#cccccc',
        },
      },
    }"
  >
    <a-app>
      <DefaultLayout>
        <router-view v-slot="{ Component, route }">
          <transition name="page" mode="out-in">
            <component :is="Component" :key="route.fullPath" />
          </transition>
        </router-view>
      </DefaultLayout>
    </a-app>
  </a-config-provider>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { theme } from 'ant-design-vue'
import DefaultLayout from './layouts/DefaultLayout.vue'
import { useTheme } from './composables/useTheme'

const { theme: themeMode } = useTheme()
const isDark = computed(() => themeMode.value === 'dark')
</script>

<style>
/* global tokens live in src/styles/global.css */
</style>
